//! Action items: what the team agreed to work on, grouped by status. Items
//! come from post-game reviews or the form at the top; each has an owner
//! (a username) and an optional due day. Rows edit in place, delete with
//! Undo, and work from the keyboard (see `KEYS` in the page legend).

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, Checkbox, Density, EmptySize,
    EmptyState, Field, InputKind, Keycap, NoTeamState, PageHeader, PageShell, SectionHeader,
    Select, SelectOption, Skeleton, SkeletonGroup, SkeletonShape, TextField, Toast, ToastContext,
};
use crate::models::action_item::ActionItem;
use leptos::prelude::*;

/// Everything the page shows, in one request.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ActionItemsData {
    pub has_team: bool,
    pub items: Vec<ActionItem>,
    /// Usernames of the team's members (the owner choices).
    pub members: Vec<String>,
    /// The server's date, `YYYY-MM-DD`, for "overdue".
    pub today: String,
}

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_action_items() -> Result<ActionItemsData, ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let Some(team_id) = ctx.team().await? else {
        return Ok(ActionItemsData {
            has_team: false,
            items: Vec::new(),
            members: Vec::new(),
            today,
        });
    };
    let items = db::list_action_items(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let members = db::get_user_team_with_members(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)?
        .map(|(_, m)| m.into_iter().map(|m| m.username).collect())
        .unwrap_or_default();
    Ok(ActionItemsData {
        has_team: true,
        items,
        members,
        today,
    })
}

#[server]
pub async fn create_action_item_fn(
    text: String,
    assigned_to: Option<String>,
    due: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let assigned = assigned_to.filter(|s| !s.is_empty());
    db::create_action_item(&ctx.db, &team_id, text, None, assigned, due)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_item_status(id: String, status: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::update_action_item_status(&ctx.db, &team_id, &id, status)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_item(
    id: String,
    text: String,
    assigned_to: Option<String>,
    due: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let assigned = assigned_to.filter(|s| !s.is_empty());
    // The status is left as it is (it changes through update_item_status).
    db::update_action_item(&ctx.db, &team_id, &id, text, assigned, due, None)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_item(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::delete_action_item(&ctx.db, &team_id, &id)
        .await
        .map_err(authz::server_err)
}

/// Undo of a delete: the item again, with its status and review link (the
/// review is checked against the team).
#[server]
pub async fn restore_item(
    text: String,
    assigned_to: Option<String>,
    due: Option<String>,
    status: String,
    source_review: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let id = db::create_action_item(&ctx.db, &team_id, text, source_review, assigned_to, due)
        .await
        .map_err(authz::server_err)?;
    if status != "open" {
        db::update_action_item_status(&ctx.db, &team_id, &id, status)
            .await
            .map_err(authz::server_err)?;
    }
    Ok(id)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Status groups in page order: (status, heading).
const GROUPS: [(&str, &str); 3] = [
    ("in_progress", "In progress"),
    ("open", "Open"),
    ("done", "Done"),
];

/// DOM id of an item's checkbox (the row's keyboard anchor).
fn item_dom_id(id: &str) -> String {
    let key: String = id
        .rsplit(':')
        .next()
        .unwrap_or(id)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    format!("ai-{key}")
}

/// "Due 31 Oct" (with the year when it is not this year's).
fn due_label(due: &str, today: &str) -> String {
    match chrono::NaiveDate::parse_from_str(due, "%Y-%m-%d") {
        Ok(d) if today.get(..4) == Some(&due[..4]) => format!("Due {}", d.format("%-d %b")),
        Ok(d) => format!("Due {}", d.format("%-d %b %Y")),
        Err(_) => format!("Due {due}"),
    }
}

/// Items of one status, in the order the group shows them: overdue first,
/// then by due day (none last), then newest; done items newest first.
fn group_items(items: &[ActionItem], status: &str, owner: &str) -> Vec<ActionItem> {
    let mut v: Vec<ActionItem> = items
        .iter()
        .filter(|i| i.status == status)
        .filter(|i| match owner {
            "" => true,
            "unassigned" => i.assigned_to.is_none(),
            name => i.assigned_to.as_deref() == Some(name),
        })
        .cloned()
        .collect();
    if status == "done" {
        v.sort_by(|a, b| b.resolved_at.cmp(&a.resolved_at));
    } else {
        v.sort_by(|a, b| match (&a.due, &b.due) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.created_at.cmp(&a.created_at),
        });
    }
    v
}

/// The id `step` places from `from` in `order` (the visible rows).
fn neighbour(order: &[String], from: &str, step: i32) -> Option<String> {
    let pos = order.iter().position(|id| id == from)? as i32 + step;
    usize::try_from(pos)
        .ok()
        .and_then(|p| order.get(p))
        .cloned()
}

/// Focus an element by id once the DOM has caught up.
#[cfg(feature = "hydrate")]
fn focus_later(id: String) {
    use wasm_bindgen::prelude::*;
    let cb = Closure::once(move || {
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&id))
        {
            if let Ok(el) = el.dyn_into::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
        }
    });
    if let Some(win) = web_sys::window() {
        let _ = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 50);
    }
    cb.forget();
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

/// Shared by the rows: refresh the list, then put focus on an element.
#[derive(Clone, Copy)]
struct Page {
    data: Resource<Result<ActionItemsData, ServerFnError>>,
    /// Element id to focus after the next load (a moved or restored row).
    focus_after: RwSignal<Option<String>>,
    /// The row being edited (item id).
    editing: RwSignal<Option<String>>,
    /// Checkbox ids of the visible rows, top to bottom (arrow keys).
    order: RwSignal<Vec<String>>,
}

#[component]
pub fn ActionItemsPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let data = Resource::new(|| (), |_| get_action_items());
    let page = Page {
        data,
        focus_after: RwSignal::new(None),
        editing: RwSignal::new(None),
        order: RwSignal::new(Vec::new()),
    };
    provide_context(page);
    let owner_filter = RwSignal::new(String::new());

    // After a reload, focus the row that moved (or was restored).
    Effect::new(move |_| {
        if let Some(Ok(_)) = data.get() {
            if let Some(id) = page.focus_after.get_untracked() {
                page.focus_after.set(None);
                #[cfg(feature = "hydrate")]
                focus_later(id);
                #[cfg(not(feature = "hydrate"))]
                let _ = id;
            }
        }
    });

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Action items">
            <PageHeader
                title="Action items"
                eyebrow="Team"
                meta=move || view! {
                    <Suspense fallback=|| ()>
                        {move || data.get().and_then(|r| r.ok()).filter(|d| d.has_team).map(|d| {
                            let count = |s: &str| d.items.iter().filter(|i| i.status == s).count();
                            let overdue = d.items.iter().filter(|i| i.is_overdue(&d.today)).count();
                            view! {
                                <span class="figures" data-testid="action-counts">
                                    {format!("{} open \u{b7} {} in progress \u{b7} {} done", count("open"), count("in_progress"), count("done"))}
                                </span>
                                {(overdue > 0).then(|| view! { <span class="text-warning">{format!("{overdue} overdue")}</span> })}
                            }
                        })}
                    </Suspense>
                }
            />
            // A transition keeps the list on screen while it reloads.
            <Transition fallback=|| view! { <SkeletonGroup label="Loading action items"><Skeleton shape=SkeletonShape::Line lines=6 /></SkeletonGroup> }>
                {move || data.get().map(|r| match r {
                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"The action items did not load."</Callout> }.into_any(),
                    Ok(d) if !d.has_team => view! { <NoTeamState /> }.into_any(),
                    Ok(d) => view! { <ItemsBody data=d owner_filter=owner_filter /> }.into_any(),
                })}
            </Transition>
        </PageShell>
    }
}

#[component]
fn ItemsBody(data: ActionItemsData, owner_filter: RwSignal<String>) -> impl IntoView {
    let members = StoredValue::new(data.members.clone());
    let items = StoredValue::new(data.items.clone());
    let today = StoredValue::new(data.today.clone());
    let total = data.items.len();
    let page = expect_context::<Page>();
    Effect::new(move |_| {
        let f = owner_filter.get();
        let order = items.with_value(|all| {
            GROUPS
                .iter()
                .flat_map(|(s, _)| group_items(all, s, &f))
                .map(|i| item_dom_id(i.id.as_deref().unwrap_or_default()))
                .collect()
        });
        page.order.set(order);
    });

    let mut filter_options = vec![
        SelectOption::new("", "Everyone"),
        SelectOption::new("unassigned", "Unassigned"),
    ];
    filter_options.extend(
        data.members
            .iter()
            .map(|m| SelectOption::new(m.clone(), m.clone())),
    );

    view! {
        <div class="flex flex-col gap-10">
            <NewItemForm members=data.members.clone() />
            {if total == 0 {
                view! {
                    <EmptyState
                        title="No action items yet"
                        message="Items appear here when a post-game review lists them under Action items, or when you add one above. Each has an owner and can have a due day."
                        cta_label="Write a review"
                        cta_href="/post-game"
                    />
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col gap-8">
                        <div class="flex flex-wrap items-end justify-between gap-4">
                            <div class="w-56">
                                <Field id="ai-owner-filter" label="Show items for">
                                    <Select value=owner_filter options=filter_options />
                                </Field>
                            </div>
                            <p class="flex flex-wrap items-center gap-x-3 gap-y-1.5 text-caption text-muted" data-action-keys="">
                                <span class="inline-flex items-center gap-1"><Keycap keys="\u{2191}" /><Keycap keys="\u{2193}" />" move"</span>
                                <span class="inline-flex items-center gap-1"><Keycap keys="Space" />" done"</span>
                                <span class="inline-flex items-center gap-1"><Keycap keys="E" />" edit"</span>
                                <span class="inline-flex items-center gap-1"><Keycap keys="Del" />" delete"</span>
                            </p>
                        </div>
                        {GROUPS.iter().map(|(status, heading)| {
                            let status = *status;
                            let heading_id = format!("ai-group-{status}");
                            view! {
                                {move || {
                                    let list = items.with_value(|all| group_items(all, status, &owner_filter.get()));
                                    let heading_id = heading_id.clone();
                                    let labelled_by = heading_id.clone();
                                    (!list.is_empty()).then(|| view! {
                                        <section aria-labelledby=labelled_by class="flex flex-col gap-2" data-group=status>
                                            <SectionHeader title=heading.to_string() id=heading_id meta=format!("{}", list.len()) />
                                            <ul class="flex flex-col border-t border-rule">
                                                {list.into_iter().map(|item| view! {
                                                    <ItemRow item=item members=members.get_value() today=today.get_value() />
                                                }).collect_view()}
                                            </ul>
                                        </section>
                                    })
                                }}
                            }
                        }).collect_view()}
                        {move || {
                            let f = owner_filter.get();
                            let none = items.with_value(|all| GROUPS.iter().all(|(s, _)| group_items(all, s, &f).is_empty()));
                            none.then(|| view! {
                                <EmptyState size=EmptySize::Compact message="Nobody's items match this filter. Choose Everyone to see the whole list." />
                            })
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

/// Text, owner and due day; Enter in the text adds the item.
#[component]
fn NewItemForm(members: Vec<String>) -> impl IntoView {
    let page = expect_context::<Page>();
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let text = RwSignal::new(String::new());
    let owner = RwSignal::new(String::new());
    let due = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let due_error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);

    let add = Callback::new(move |_: ()| {
        let t = text.get_untracked().trim().to_string();
        if t.is_empty() {
            error.set(Some("Write what needs doing.".into()));
            return;
        }
        error.set(None);
        let d = due.get_untracked();
        if let Err(e) = ActionItem::parse_due(&d) {
            due_error.set(Some(e.into()));
            return;
        }
        due_error.set(None);
        let o = owner.get_untracked();
        saving.set(true);
        leptos::task::spawn_local(async move {
            match create_action_item_fn(t, Some(o), Some(d)).await {
                Ok(id) => {
                    text.set(String::new());
                    due.set(String::new());
                    page.focus_after.set(Some(item_dom_id(&id)));
                    page.data.refetch();
                    toast.push.run(Toast::success("Action item added"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not add the item: {e}"))),
            }
            saving.set(false);
        });
    });

    let mut owner_options = vec![SelectOption::new("", "Unassigned")];
    owner_options.extend(members.into_iter().map(|m| SelectOption::new(m.clone(), m)));

    view! {
        <section aria-labelledby="ai-new-heading" class="flex flex-col gap-3">
            <SectionHeader title="New item" id="ai-new-heading" />
            <div class="grid grid-cols-1 items-start gap-3 md:grid-cols-[minmax(0,1fr)_12rem_11rem_auto]">
                <Field id="ai-new-text" label="What needs doing" error=Signal::derive(move || error.get())>
                    <TextField
                        value=text
                        placeholder="Ward the enemy jungle before 3:15"
                        on_keydown=Callback::new(move |ev: leptos::ev::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                ev.prevent_default();
                                add.run(());
                            }
                        })
                    />
                </Field>
                <Field id="ai-new-owner" label="Owner">
                    <Select value=owner options=owner_options />
                </Field>
                <Field id="ai-new-due" label="Due" error=Signal::derive(move || due_error.get())>
                    <TextField value=due kind=InputKind::Date />
                </Field>
                <div class="md:pt-6">
                    <Button
                        variant=ButtonVariant::Primary
                        loading=Signal::derive(move || saving.get())
                        on_click=Callback::new(move |_| add.run(()))
                        attr:data-testid="ai-add"
                    >
                        "Add item"
                    </Button>
                </div>
            </div>
        </section>
    }
}

/// One item: a checkbox (done), owner, due day, and its actions. `E` edits
/// in place, `Delete` removes it (with Undo), arrows move between rows.
#[component]
fn ItemRow(item: ActionItem, members: Vec<String>, today: String) -> impl IntoView {
    let page = expect_context::<Page>();
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let id = item.id.clone().unwrap_or_default();
    let dom_id = item_dom_id(&id);
    let item_sv = StoredValue::new(item.clone());
    let id_sv = StoredValue::new(id.clone());
    let dom_sv = StoredValue::new(dom_id.clone());
    let done = RwSignal::new(item.status == "done");
    let editing =
        Memo::new(move |_| page.editing.get().as_deref() == Some(id_sv.get_value().as_str()));

    let set_status = Callback::new(move |status: &'static str| {
        let id = id_sv.get_value();
        leptos::task::spawn_local(async move {
            match update_item_status(id, status.to_string()).await {
                Ok(()) => {
                    page.focus_after.set(Some(dom_sv.get_value()));
                    page.data.refetch();
                    let msg = match status {
                        "done" => "Marked done",
                        "in_progress" => "Moved to in progress",
                        _ => "Moved to open",
                    };
                    toast.push.run(Toast::success(msg));
                }
                Err(e) => {
                    done.set(item_sv.with_value(|i| i.status == "done"));
                    toast
                        .push
                        .run(Toast::error(format!("Could not update the item: {e}")));
                }
            }
        });
    });

    let remove = Callback::new(move |_: ()| {
        let it = item_sv.get_value();
        let id = id_sv.get_value();
        leptos::task::spawn_local(async move {
            match delete_item(id).await {
                Ok(()) => {
                    page.data.refetch();
                    let undo = Callback::new(move |_| {
                        let it = it.clone();
                        leptos::task::spawn_local(async move {
                            match restore_item(
                                it.text.clone(),
                                it.assigned_to.clone(),
                                it.due.clone(),
                                it.status.clone(),
                                it.source_review.clone(),
                            )
                            .await
                            {
                                Ok(new_id) => {
                                    page.focus_after.set(Some(item_dom_id(&new_id)));
                                    page.data.refetch();
                                    toast.push.run(Toast::success("Item restored"));
                                }
                                Err(e) => toast
                                    .push
                                    .run(Toast::error(format!("Could not restore it: {e}"))),
                            }
                        });
                    });
                    toast
                        .push
                        .run(Toast::info("Action item deleted").with_action("Undo", undo));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });

    // Row keys apply when focus is on the row's checkbox or buttons, never
    // while typing in a field.
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if editing.get_untracked() {
            if ev.key() == "Escape" {
                ev.prevent_default();
                page.editing.set(None);
                #[cfg(feature = "hydrate")]
                focus_later(dom_sv.get_value());
            }
            return;
        }
        match ev.key().as_str() {
            "ArrowDown" | "ArrowUp" => {
                ev.prevent_default();
                let step = if ev.key() == "ArrowDown" { 1 } else { -1 };
                if let Some(next) = page
                    .order
                    .with_untracked(|o| neighbour(o, &dom_sv.get_value(), step))
                {
                    #[cfg(feature = "hydrate")]
                    focus_later(next);
                    #[cfg(not(feature = "hydrate"))]
                    let _ = next;
                }
            }
            "e" | "E" if !ev.ctrl_key() && !ev.meta_key() && !ev.alt_key() => {
                ev.prevent_default();
                page.editing.set(Some(id_sv.get_value()));
            }
            "Delete" => {
                ev.prevent_default();
                // Focus lands on the next row (or the previous at the end).
                let next = page.order.with_untracked(|o| {
                    neighbour(o, &dom_sv.get_value(), 1)
                        .or_else(|| neighbour(o, &dom_sv.get_value(), -1))
                });
                page.focus_after.set(next);
                remove.run(());
            }
            _ => {}
        }
    };

    let status = item.status.clone();
    let status_attr = item.status.clone();
    let overdue = item.is_overdue(&today);
    let due_text = item.due.as_deref().map(|d| due_label(d, &today));
    let owner = item.assigned_to.clone();
    let review = item.source_review.clone();
    let members = StoredValue::new(members);

    view! {
        <li
            class="border-b border-rule py-3"
            data-action-item=id.clone()
            data-status=status_attr
            on:keydown=on_keydown
        >
            {move || if editing.get() {
                view! { <ItemEditor item=item_sv.get_value() members=members.get_value() /> }.into_any()
            } else {
                let status = status.clone();
                let dom_id = dom_id.clone();
                let due_text = due_text.clone();
                let owner = owner.clone();
                let review = review.clone();
                view! {
                    <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-6">
                        <div class="flex min-w-0 flex-col gap-1">
                            <Checkbox
                                id=dom_id
                                label=item_sv.with_value(|i| i.text.clone())
                                checked=done
                                strike=true
                                on_change=Callback::new(move |v: bool| set_status.run(if v { "done" } else { "open" }))
                            />
                            <p class="flex flex-wrap gap-x-3 gap-y-0.5 pl-6.5 text-caption text-muted">
                                <span data-owner="">{match owner { Some(o) => format!("Owner {o}"), None => "Unassigned".to_string() }}</span>
                                {due_text.map(|d| view! {
                                    <span class=if overdue { "text-warning" } else { "" } data-due="">
                                        {if overdue { format!("Overdue \u{b7} {d}") } else { d }}
                                    </span>
                                })}
                                {review.map(|r| view! {
                                    <a class="underline decoration-rule underline-offset-2 hover:text-primary" href=format!("/post-game?review_id={r}")>"From a review"</a>
                                })}
                            </p>
                        </div>
                        <div class="flex shrink-0 flex-wrap items-center gap-1 pl-6.5 sm:pl-0">
                            {match status.as_str() {
                                "open" => Some(view! {
                                    <Button size=ButtonSize::Sm on_click=Callback::new(move |_| set_status.run("in_progress"))>"Start"</Button>
                                }.into_any()),
                                "in_progress" => Some(view! {
                                    <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| set_status.run("open"))>"Back to open"</Button>
                                }.into_any()),
                                _ => None,
                            }}
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| page.editing.set(Some(id_sv.get_value())))>"Edit"</Button>
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| remove.run(()))>"Delete"</Button>
                        </div>
                    </div>
                }.into_any()
            }}
        </li>
    }
}

/// The row in edit mode: Enter saves, Escape cancels (handled by the row).
#[component]
fn ItemEditor(item: ActionItem, members: Vec<String>) -> impl IntoView {
    let page = expect_context::<Page>();
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let id = StoredValue::new(item.id.clone().unwrap_or_default());
    let text = RwSignal::new(item.text.clone());
    let owner = RwSignal::new(item.assigned_to.clone().unwrap_or_default());
    let due = RwSignal::new(item.due.clone().unwrap_or_default());
    let error = RwSignal::new(None::<String>);
    let due_error = RwSignal::new(None::<String>);
    let key = item_dom_id(&id.get_value());
    let text_id = format!("{key}-text");

    let save = Callback::new(move |_: ()| {
        let t = text.get_untracked().trim().to_string();
        if t.is_empty() {
            error.set(Some("The item needs some text.".into()));
            return;
        }
        let d = due.get_untracked();
        if let Err(e) = ActionItem::parse_due(&d) {
            due_error.set(Some(e.into()));
            return;
        }
        let o = owner.get_untracked();
        let item_id = id.get_value();
        leptos::task::spawn_local(async move {
            match update_item(item_id.clone(), t, Some(o), Some(d)).await {
                Ok(()) => {
                    page.editing.set(None);
                    page.focus_after.set(Some(item_dom_id(&item_id)));
                    page.data.refetch();
                    toast.push.run(Toast::success("Item saved"));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    });
    let on_enter = Callback::new(move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            save.run(());
        }
    });

    #[cfg(feature = "hydrate")]
    focus_later(text_id.clone());

    let mut owner_options = vec![SelectOption::new("", "Unassigned")];
    owner_options.extend(members.into_iter().map(|m| SelectOption::new(m.clone(), m)));

    view! {
        <div class="grid grid-cols-1 items-start gap-3 md:grid-cols-[minmax(0,1fr)_12rem_11rem]" data-item-editor="">
            <Field id=text_id label="Item" error=Signal::derive(move || error.get())>
                <TextField value=text on_keydown=on_enter />
            </Field>
            <Field id=format!("{key}-owner") label="Owner">
                <Select value=owner options=owner_options />
            </Field>
            <Field id=format!("{key}-due") label="Due" error=Signal::derive(move || due_error.get())>
                <TextField value=due kind=InputKind::Date on_keydown=on_enter />
            </Field>
            <div class="flex gap-2 md:col-span-3">
                <Button size=ButtonSize::Sm variant=ButtonVariant::Primary on_click=Callback::new(move |_| save.run(()))>"Save"</Button>
                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| page.editing.set(None))>"Cancel"</Button>
                <span class="self-center text-caption text-muted">"Enter saves, Esc cancels"</span>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(status: &str, due: Option<&str>, created: &str, owner: Option<&str>) -> ActionItem {
        ActionItem {
            id: Some(format!("action_item:{created}")),
            team_id: "team:t".into(),
            source_review: None,
            text: created.into(),
            status: status.into(),
            assigned_to: owner.map(Into::into),
            due: due.map(Into::into),
            created_at: Some(created.into()),
            resolved_at: None,
        }
    }

    #[test]
    fn groups_sort_by_due_day_then_newest() {
        let items = vec![
            item("open", None, "a", None),
            item("open", Some("2026-10-20"), "b", Some("mia")),
            item("open", Some("2026-10-12"), "c", None),
            item("open", None, "d", None),
            item("done", None, "e", None),
        ];
        let order: Vec<String> = group_items(&items, "open", "")
            .into_iter()
            .map(|i| i.text)
            .collect();
        assert_eq!(order, ["c", "b", "d", "a"]);
        assert_eq!(group_items(&items, "open", "mia").len(), 1);
        assert_eq!(group_items(&items, "open", "unassigned").len(), 3);
        assert_eq!(group_items(&items, "done", "").len(), 1);
    }

    #[test]
    fn dom_ids_use_the_record_key() {
        assert_eq!(item_dom_id("action_item:x7k2"), "ai-x7k2");
        assert_eq!(item_dom_id("action_item:a-b"), "ai-ab");
    }

    #[test]
    fn arrow_neighbours_stop_at_the_ends() {
        let order: Vec<String> = ["ai-a", "ai-b", "ai-c"].map(String::from).to_vec();
        assert_eq!(neighbour(&order, "ai-b", 1).as_deref(), Some("ai-c"));
        assert_eq!(neighbour(&order, "ai-a", -1), None);
        assert_eq!(neighbour(&order, "ai-c", 1), None);
        assert_eq!(neighbour(&order, "ai-x", 1), None);
    }

    #[test]
    fn due_labels_drop_this_year() {
        assert_eq!(due_label("2026-10-31", "2026-10-11"), "Due 31 Oct");
        assert_eq!(due_label("2027-01-05", "2026-10-11"), "Due 5 Jan 2027");
    }
}
