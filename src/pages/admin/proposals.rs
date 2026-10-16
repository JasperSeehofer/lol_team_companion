//! `/admin/proposals`: What next proposals (M7). Admins write and move
//! ideas, link shipped ones to a changelog entry, and publish them: nothing
//! reaches the board before it is published (the seeded ideas start
//! unpublished, for the owner to review).

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::components::ui::{
    Badge, BadgeTone, Button, ButtonVariant, Callout, CalloutTone, Checkbox, ConfirmDialog,
    ConfirmTone, Dialog, DialogSize, EmptySize, EmptyState, Field, PageHeader, PageShell,
    PageWidth, Select, SelectOption, Skeleton, SkeletonGroup, TextField, ToastContext, ToastKind,
};
use crate::error_template::NotFound;
use crate::models::community::{
    agent_attribution, parse_proposal_import, proposal_source_label, votes_label, ChangelogChoice,
    ProposalInput, ProposalStatus, ProposalView,
};
use crate::pages::admin::bugs::err_text;
use crate::pages::admin::ListRow;
use crate::pages::community::ProposalStatusBadge;

#[server]
pub async fn admin_list_proposals(
) -> Result<(Vec<ProposalView>, Vec<ChangelogChoice>), ServerFnError> {
    use crate::server::{authz, community::changelog, community::store};
    authz::require_admin().await?;
    let db = authz::db()?;
    let list = store::admin_list_proposals(&db)
        .await
        .map_err(authz::server_err)?;
    Ok((list, changelog::choices()))
}

#[server]
pub async fn admin_save_proposal(proposal_json: String) -> Result<String, ServerFnError> {
    use crate::server::{authz, community::changelog, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let input: ProposalInput = serde_json::from_str(&proposal_json).map_err(authz::server_err)?;
    let input =
        store::validate_proposal(&input, changelog::entry_exists).map_err(authz::server_err)?;
    let id = store::admin_save_proposal(&db, &admin.id, &input)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, proposal = %id, status = %input.status, published = input.published, "proposal saved");
    crate::server::bug_report_export::request_export();
    Ok(id)
}

/// Import agent proposals from JSON (M141 H4; schema in CLAUDE.md): admins
/// only, every proposal a draft that members never see until it is
/// published. Returns how many were imported and the date (YYYY-MM-DD).
#[server]
pub async fn admin_import_proposals(json: String) -> Result<(usize, String), ServerFnError> {
    use crate::models::community::parse_proposal_import;
    use crate::server::rate_limit::Bucket;
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    authz::rate_limit_user(Bucket::Community, &admin)
        .map_err(|_| ServerFnError::new("Too many imports in a short time. Wait a minute."))?;
    let items = parse_proposal_import(&json).map_err(|errs| ServerFnError::new(errs.join("\n")))?;
    let db = authz::db()?;
    let n = store::admin_import_proposals(&db, &admin.id, &items)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, count = n, "agent proposals imported as drafts");
    crate::server::bug_report_export::request_export();
    Ok((n, chrono::Utc::now().format("%Y-%m-%d").to_string()))
}

#[server]
pub async fn admin_delete_proposal(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    store::admin_delete_proposal(&db, &id)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, proposal = %id, "proposal deleted");
    crate::server::bug_report_export::request_export();
    Ok(())
}

#[component]
pub fn AdminProposalsPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| admin_list_proposals());
    let query = use_query_map();
    let selected = Signal::derive(move || query.get().get("id"));
    let import_open = RwSignal::new(false);
    let imported = RwSignal::new(None::<String>);
    view! {
        {move || imported.get().map(|msg| view! {
            <div class="mx-auto w-full max-w-7xl px-4 pt-6 sm:px-8" data-testid="import-result">
                <Callout tone=CalloutTone::Success>{msg}</Callout>
            </div>
        })}
        <ImportDialog open=import_open on_done=Callback::new(move |msg: String| {
            imported.set(Some(msg));
            data.refetch();
        }) />
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading proposals"><Skeleton lines=6 /></SkeletonGroup></PageShell> }>
            {move || data.get().map(|r| match r {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((list, choices)) => view! {
                    <PageShell width=PageWidth::Wide>
                        <PageHeader
                            title="What next proposals"
                            eyebrow="Admin"
                            meta=|| view! { <span>"Ideas on the board, their status, and the release that shipped them. Users see only published ideas."</span> }
                            actions=move || view! {
                                <Button href="/admin" variant=ButtonVariant::Quiet>"All admin areas"</Button>
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| import_open.set(true))>"Import proposals (JSON)"</Button>
                                <Button href="/admin/proposals?id=new" variant=ButtonVariant::Secondary>"New proposal"</Button>
                            }
                        />
                        <Proposals list=list choices=choices selected=selected on_saved=Callback::new(move |_| data.refetch()) />
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn Proposals(
    list: Vec<ProposalView>,
    choices: Vec<ChangelogChoice>,
    selected: Signal<Option<String>>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let list = StoredValue::new(list);
    let choices = StoredValue::new(choices);
    view! {
        <div class="grid gap-8 lg:grid-cols-[minmax(0,24rem)_minmax(0,1fr)]">
            <div class="min-w-0">
                {if list.get_value().is_empty() {
                    view! { <EmptyState size=EmptySize::Compact message="No proposals yet. Add one with New proposal, or make one from a wishlist report." /> }.into_any()
                } else {
                    view! {
                        <ul class="flex flex-col border-t border-rule" aria-label="Proposals">
                            {list.get_value().into_iter().map(|p| {
                                let id = p.id.clone();
                                let is_sel = Signal::derive({
                                    let id = id.clone();
                                    move || selected.get().as_deref() == Some(id.as_str())
                                });
                                view! {
                                    <li class="border-b border-rule">
                                        <ListRow href=format!("/admin/proposals?id={id}") selected=is_sel>
                                            <span class="flex w-full flex-col items-start gap-1 py-1 text-left">
                                                <span class="text-body text-primary">{p.title.clone()}</span>
                                                <span class="flex flex-wrap items-center gap-2 text-caption text-muted">
                                                    <ProposalStatusBadge status=p.status.clone() />
                                                    {if p.published {
                                                        view! { <Badge tone=BadgeTone::Success>"Published"</Badge> }.into_any()
                                                    } else {
                                                        view! { <Badge tone=BadgeTone::Warning>"Not published"</Badge> }.into_any()
                                                    }}
                                                    <span class="figures">{votes_label(p.votes)}</span>
                                                </span>
                                                {p.agent_proposed_on.clone().map(|d| view! {
                                                    <span class="text-caption text-muted">{agent_attribution(&d)}</span>
                                                })}
                                            </span>
                                        </ListRow>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </div>
            <div class="min-w-0">
                {move || {
                    let sel = selected.get();
                    match sel.as_deref() {
                        Some("new") => view! { <Editor p=None choices=choices.get_value() on_saved=on_saved /> }.into_any(),
                        Some(id) => match list.get_value().into_iter().find(|p| p.id == id) {
                            Some(p) => view! { <Editor p=Some(p) choices=choices.get_value() on_saved=on_saved /> }.into_any(),
                            None => view! { <EmptyState size=EmptySize::Compact message="That proposal no longer exists." /> }.into_any(),
                        },
                        None => view! { <EmptyState size=EmptySize::Compact message="Choose a proposal from the list, or write a new one." /> }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn Editor(
    p: Option<ProposalView>,
    choices: Vec<ChangelogChoice>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let navigate = StoredValue::new(use_navigate());
    let existing = p.clone();
    let id = StoredValue::new(p.as_ref().map(|p| p.id.clone()));
    let title = RwSignal::new(p.as_ref().map(|p| p.title.clone()).unwrap_or_default());
    let summary = RwSignal::new(p.as_ref().map(|p| p.summary.clone()).unwrap_or_default());
    let status = RwSignal::new(
        p.as_ref()
            .map(|p| p.status.clone())
            .unwrap_or_else(|| "proposed".into()),
    );
    let note = RwSignal::new(
        p.as_ref()
            .and_then(|p| p.status_note.clone())
            .unwrap_or_default(),
    );
    let shipped_in = RwSignal::new(
        p.as_ref()
            .and_then(|p| p.shipped_in.clone())
            .or_else(|| choices.first().map(|c| c.0.clone()))
            .unwrap_or_default(),
    );
    let published = RwSignal::new(p.as_ref().is_some_and(|p| p.published));
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let saving = RwSignal::new(false);
    let confirm_delete = RwSignal::new(false);
    let votes = p.as_ref().map(|p| p.votes).unwrap_or(0);

    let save = move |_| {
        error.set(None);
        let input = ProposalInput {
            id: id.get_value(),
            title: title.get_untracked(),
            summary: summary.get_untracked(),
            status: status.get_untracked(),
            status_note: note.get_untracked(),
            shipped_in: shipped_in.get_untracked(),
            published: published.get_untracked(),
        };
        let json = serde_json::to_string(&input).unwrap_or_default();
        saving.set(true);
        spawn_local(async move {
            let r = admin_save_proposal(json).await;
            saving.set(false);
            match r {
                Ok(new_id) => {
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Success, "Proposal saved.".into()));
                    }
                    let url = format!("/admin/proposals?id={new_id}");
                    navigate.with_value(|n| {
                        n(
                            &url,
                            leptos_router::NavigateOptions {
                                replace: true,
                                scroll: false,
                                ..Default::default()
                            },
                        )
                    });
                    on_saved.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    };
    let delete = Callback::new(move |_| {
        let Some(pid) = id.get_value() else { return };
        spawn_local(async move {
            match admin_delete_proposal(pid).await {
                Ok(()) => {
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Success, "Proposal deleted.".into()));
                    }
                    navigate.with_value(|n| n("/admin/proposals", Default::default()));
                    on_saved.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    });
    let status_options: Vec<SelectOption> = ProposalStatus::ALL
        .iter()
        .map(|s| SelectOption::new(s.as_str(), s.label()))
        .collect();
    let entry_options: Vec<SelectOption> = choices
        .into_iter()
        .map(|(slug, t)| SelectOption::new(slug, t))
        .collect();
    let entry_options = StoredValue::new(entry_options);
    view! {
        <div class="flex flex-col gap-6" data-testid="proposal-editor">
            {existing.as_ref().map(|p| view! {
                <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-x-4 gap-y-2">
                    <dt class="text-caption text-muted">"Source"</dt>
                    <dd class="text-body text-primary">{match p.agent_proposed_on.clone() {
                        Some(d) => agent_attribution(&d),
                        None => proposal_source_label(&p.source).to_string(),
                    }}</dd>
                    <dt class="text-caption text-muted">"Votes"</dt>
                    <dd class="figures text-body text-primary">{p.votes}</dd>
                    {p.wishlist_report.clone().map(|r| view! {
                        <dt class="text-caption text-muted">"Report"</dt>
                        <dd><a class="text-body text-primary underline decoration-rule underline-offset-4" href=format!("/admin/bugs?id={r}")>"Open the wishlist report"</a></dd>
                    })}
                </dl>
            })}
            <Field id="proposal-title" label="Title">
                <TextField value=title maxlength=100 />
            </Field>
            <Field id="proposal-summary" label="Summary" hint="What it is and who it helps, in two or three sentences. Up to 1,000 characters.">
                <TextField value=summary rows=6 maxlength=1000 />
            </Field>
            <Field id="proposal-status" label="Status">
                <Select value=status options=status_options />
            </Field>
            <Field id="proposal-note" label="Status note" hint="Shown to everyone. Required when you decline an idea.">
                <TextField value=note rows=2 maxlength=600 />
            </Field>
            {move || (status.get() == "shipped").then(|| view! {
                <Field id="proposal-shipped" label="Shipped in">
                    <Select value=shipped_in options=entry_options.get_value() />
                </Field>
            })}
            <Checkbox label="Published (shown on What next)" checked=published />
            {move || error.get().map(|e| view! { <p class="text-label text-danger" role="alert">{e}</p> })}
            <div class="flex flex-wrap items-center gap-3">
                <Button variant=ButtonVariant::Primary loading=saving on_click=Callback::new(save)>"Save"</Button>
                {(existing.is_some() && votes == 0).then(|| view! {
                    <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| confirm_delete.set(true))>"Delete"</Button>
                })}
            </div>
            <ConfirmDialog
                id="proposal-delete"
                open=confirm_delete
                title="Delete this proposal?"
                message="It has no votes. This cannot be undone."
                confirm_label="Delete"
                tone=ConfirmTone::Destructive
                on_confirm=delete
            />
        </div>
    }
}

/// "Import proposals (JSON)": paste or load a proposals file (schema in
/// CLAUDE.md); everything imported is a draft until published one by one.
#[component]
fn ImportDialog(open: RwSignal<bool>, on_done: Callback<String>) -> impl IntoView {
    let json = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let parsed = Memo::new(move |_| {
        let text = json.get();
        if text.trim().is_empty() {
            None
        } else {
            Some(parse_proposal_import(&text).map(|l| l.len()))
        }
    });
    let import = Callback::new(move |_: ()| {
        if busy.get_untracked() || !matches!(parsed.get_untracked(), Some(Ok(_))) {
            return;
        }
        busy.set(true);
        error.set(None);
        let body = json.get_untracked();
        spawn_local(async move {
            let r = admin_import_proposals(body).await;
            busy.set(false);
            match r {
                Ok((n, today)) => {
                    let word = if n == 1 { "draft" } else { "drafts" };
                    json.set(String::new());
                    open.set(false);
                    on_done.run(format!(
                        "{n} {word} imported, proposed by the Team Companion agent on {today}. Members see none of them until you publish each one."
                    ));
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    });
    view! {
        <Dialog
            id="proposal-import"
            open=open
            title="Import proposals (JSON)"
            size=DialogSize::Md
            dismiss_on_scrim=false
            footer=move || view! {
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                <Button
                    variant=ButtonVariant::Primary
                    loading=busy
                    disabled=Signal::derive(move || !matches!(parsed.get(), Some(Ok(_))))
                    on_click=Callback::new(move |_| import.run(()))
                >"Import as drafts"</Button>
            }
        >
            <div class="flex flex-col gap-4">
                <Field id="proposal-import-json" label="Proposals file" hint="Imported proposals are saved as drafts, invisible to members until you publish each one. Up to 20 per file; the format is in CLAUDE.md.">
                    <TextField value=json rows=10 placeholder="{\"version\": 1, \"proposals\": [{\"key\": \"...\", \"title\": \"...\", \"summary\": \"...\"}]}" />
                </Field>
                <p class="text-label text-secondary" data-testid="import-count">{move || match parsed.get() {
                    None => "Paste the JSON of a proposals file.".to_string(),
                    Some(Ok(1)) => "1 proposal found.".to_string(),
                    Some(Ok(n)) => format!("{n} proposals found."),
                    Some(Err(_)) => "The file has problems; nothing will be imported.".to_string(),
                }}</p>
                {move || match parsed.get() {
                    Some(Err(errs)) => Some(view! {
                        <Callout tone=CalloutTone::Danger title="Fix these first">
                            <ul class="list-disc pl-5">{errs.into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}</ul>
                        </Callout>
                    }),
                    _ => None,
                }}
                {move || error.get().map(|e| view! { <Callout tone=CalloutTone::Danger>{e}</Callout> })}
            </div>
        </Dialog>
    }
}
