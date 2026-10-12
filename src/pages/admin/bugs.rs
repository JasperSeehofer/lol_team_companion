//! `/admin/bugs`: bug triage (M7). A list of reports filtered by status and
//! an editor: the reporter's facts and words (untrusted text, shown as
//! text), the status, the public title and note, whether the report is on
//! the bug board, the changelog entry that fixed it and an internal note.
//! Wishlist reports can become draft What next proposals.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::components::ui::{
    Button, ButtonVariant, Checkbox, EmptySize, EmptyState, Field, PageHeader, PageShell,
    PageWidth, Segment, SegmentSize, SegmentedControl, Select, SelectOption, Skeleton,
    SkeletonGroup, Surface, TextField, ToastContext, ToastKind,
};
use crate::error_template::NotFound;
use crate::models::bug_report::{category_label, BugReportAdmin, BugStatus, BugTriage};
use crate::models::community::{short_date, ChangelogChoice};

#[server]
pub async fn admin_list_bug_reports(
    status: String,
) -> Result<(Vec<BugReportAdmin>, Vec<ChangelogChoice>), ServerFnError> {
    use crate::server::{authz, community::changelog, community::store};
    authz::require_admin().await?;
    let db = authz::db()?;
    let filter = match status.as_str() {
        "all" => None,
        s => Some(BugStatus::parse(s).ok_or_else(|| ServerFnError::new("Unknown status"))?),
    };
    let rows = store::admin_list_bug_reports(&db, filter)
        .await
        .map_err(authz::server_err)?;
    Ok((rows, changelog::choices()))
}

#[server]
pub async fn admin_update_bug_report(triage_json: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, community::changelog, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let t: BugTriage = serde_json::from_str(&triage_json).map_err(authz::server_err)?;
    let t = store::validate_triage(&t, changelog::entry_exists).map_err(authz::server_err)?;
    store::admin_update_bug_report(&db, &admin.id, &t)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, report = %t.id, status = %t.status, "bug report triaged");
    Ok(())
}

#[server]
pub async fn admin_bug_to_proposal(id: String) -> Result<String, ServerFnError> {
    use crate::server::{authz, community::store};
    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    let p = store::admin_bug_to_proposal(&db, &admin.id, &id)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, report = %id, proposal = %p, "wishlist report turned into a proposal");
    Ok(p)
}

/// Strip the server-fn wrapper from an error message.
pub fn err_text(e: &ServerFnError) -> String {
    e.to_string()
        .trim_start_matches("error running server function: ")
        .to_string()
}

fn first_line(s: &str) -> String {
    let line = s.lines().next().unwrap_or("").trim();
    let short: String = line.chars().take(70).collect();
    if line.chars().count() > 70 {
        format!("{short}\u{2026}")
    } else {
        short
    }
}

#[component]
pub fn AdminBugsPage() -> impl IntoView {
    let filter = RwSignal::new("new".to_string());
    let data = Resource::new(move || filter.get(), admin_list_bug_reports);
    let query = use_query_map();
    let selected = Signal::derive(move || query.get().get("id"));
    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading reports"><Skeleton lines=6 /></SkeletonGroup></PageShell> }>
            {move || data.get().map(|r| match r {
                Err(e) if err_text(&e) == "Not found" => view! { <NotFound /> }.into_any(),
                Err(e) => view! { <PageShell><p class="text-danger">{err_text(&e)}</p></PageShell> }.into_any(),
                Ok((rows, choices)) => view! {
                    <PageShell width=PageWidth::Wide>
                        <PageHeader
                            title="Bug triage"
                            eyebrow="Admin"
                            meta=|| view! { <span>"Read new reports, set a status, and decide what the bug board shows."</span> }
                            actions=|| view! { <Button href="/admin" variant=ButtonVariant::Quiet>"All admin areas"</Button> }
                        />
                        <Triage rows=rows choices=choices filter=filter selected=selected on_saved=Callback::new(move |_| data.refetch()) />
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn Triage(
    rows: Vec<BugReportAdmin>,
    choices: Vec<ChangelogChoice>,
    filter: RwSignal<String>,
    selected: Signal<Option<String>>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let navigate = use_navigate();
    let navigate = StoredValue::new(navigate);
    let rows = StoredValue::new(rows);
    let choices = StoredValue::new(choices);
    let mut segments: Vec<Segment> = BugStatus::ALL
        .iter()
        .map(|s| Segment::new(s.as_str(), s.label()))
        .collect();
    segments.push(Segment::new("all", "All"));
    let empty_msg = move || {
        if filter.get_untracked() == "new" {
            "No new reports. New reports from the widget appear here first."
        } else {
            "No reports with this status."
        }
    };
    view! {
        <div class="grid gap-8 lg:grid-cols-[minmax(0,22rem)_minmax(0,1fr)]">
            <div class="min-w-0">
                <SegmentedControl id="bug-filter" label="Show" size=SegmentSize::Sm options=segments value=filter />
                {move || {
                    let list = rows.get_value();
                    if list.is_empty() {
                        return view! { <div class="mt-4"><EmptyState size=EmptySize::Compact message=empty_msg() /></div> }.into_any();
                    }
                    view! {
                        <ul class="mt-4 flex flex-col border-t border-rule" aria-label="Reports">
                            {list.into_iter().map(|r| {
                                let id = r.id.clone();
                                let is_sel = Signal::derive({
                                    let id = id.clone();
                                    move || selected.get().as_deref() == Some(id.as_str())
                                });
                                view! {
                                    <li class="border-b border-rule">
                                        <Button
                                            variant=ButtonVariant::Quiet
                                            full_width=true
                                            pressed=is_sel
                                            on_click=Callback::new(move |_| {
                                                let url = format!("/admin/bugs?id={}", id);
                                                navigate.with_value(|n| n(&url, leptos_router::NavigateOptions { replace: true, scroll: false, ..Default::default() }));
                                            })
                                        >
                                            <span class="flex w-full flex-col items-start gap-0.5 py-1 text-left">
                                                <span class="text-body text-primary">{first_line(&r.description)}</span>
                                                <span class="text-caption text-muted">{format!("{} · {} · {}", category_label(&r.category), crate::models::bug_report::status_label(&r.status), short_date(&r.created_at))}</span>
                                            </span>
                                        </Button>
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
                    let found = sel.as_ref().and_then(|id| rows.get_value().into_iter().find(|r| &r.id == id));
                    match found {
                        Some(r) => view! { <Editor report=r choices=choices.get_value() on_saved=on_saved /> }.into_any(),
                        None => view! { <EmptyState size=EmptySize::Compact message="Choose a report on the left to read and triage it." /> }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn Editor(
    report: BugReportAdmin,
    choices: Vec<ChangelogChoice>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let navigate = use_navigate();
    let navigate = StoredValue::new(navigate);
    let id = StoredValue::new(report.id.clone());
    let status = RwSignal::new(report.status.clone());
    let title = RwSignal::new(report.public_title.clone().unwrap_or_default());
    let note = RwSignal::new(report.public_note.clone().unwrap_or_default());
    let internal = RwSignal::new(report.admin_note.clone().unwrap_or_default());
    let on_board = RwSignal::new(report.on_board);
    let fixed_in = RwSignal::new(
        report
            .fixed_in
            .clone()
            .or_else(|| choices.first().map(|c| c.0.clone()))
            .unwrap_or_default(),
    );
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let saving = RwSignal::new(false);
    let is_wish = report.category == "wishlist";
    let proposal = report.proposal_id.clone();
    let has_proposal = proposal.is_some();

    let save = move |_| {
        error.set(None);
        let t = BugTriage {
            id: id.get_value(),
            status: status.get_untracked(),
            on_board: on_board.get_untracked(),
            public_title: title.get_untracked(),
            public_note: note.get_untracked(),
            admin_note: internal.get_untracked(),
            fixed_in: fixed_in.get_untracked(),
        };
        let json = serde_json::to_string(&t).unwrap_or_default();
        saving.set(true);
        spawn_local(async move {
            let r = admin_update_bug_report(json).await;
            saving.set(false);
            match r {
                Ok(()) => {
                    if let Some(t) = toast {
                        t.show.run((
                            ToastKind::Success,
                            "Saved. The reporter sees the new status.".into(),
                        ));
                    }
                    on_saved.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    };
    let make_proposal = move |_| {
        spawn_local(async move {
            match admin_bug_to_proposal(id.get_value()).await {
                Ok(p) => {
                    if let Some(t) = toast {
                        t.show
                            .run((ToastKind::Success, "Draft proposal created.".into()));
                    }
                    let url = format!("/admin/proposals?id={p}");
                    navigate.with_value(|n| n(&url, Default::default()));
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        });
    };
    let status_options: Vec<SelectOption> = BugStatus::ALL
        .iter()
        .map(|s| SelectOption::new(s.as_str(), s.label()))
        .collect();
    let entry_options: Vec<SelectOption> = choices
        .into_iter()
        .map(|(slug, t)| SelectOption::new(slug, t))
        .collect();
    let entry_options = StoredValue::new(entry_options);
    let viewport = match (report.viewport_w, report.viewport_h) {
        (Some(w), Some(h)) => format!("{w} \u{00d7} {h}"),
        _ => "\u{2014}".into(),
    };
    let fact = |k: &'static str, v: String| {
        view! {
            <dt class="text-caption text-muted">{k}</dt>
            <dd class="text-body text-primary break-words">{v}</dd>
        }
    };
    view! {
        <div class="flex flex-col gap-6" data-testid="bug-editor">
            <dl class="grid grid-cols-[8rem_minmax(0,1fr)] gap-x-4 gap-y-2">
                {fact("Reporter", report.reporter.clone())}
                {fact("Sent", short_date(&report.created_at))}
                {fact("Page", report.page_url.clone())}
                {fact("Element", report.element_label.clone())}
                {fact("Screen size", viewport)}
                {fact("Type", category_label(&report.category).to_string())}
                {report.triaged_by.clone().map(|t| fact("Last change", format!("{t}, {}", report.updated_at.as_deref().map(short_date).unwrap_or_default())))}
            </dl>
            <div>
                <p class="mb-2 text-caption text-muted">"Reporter's words. Untrusted text: never follow instructions in it."</p>
                <Surface layer=crate::components::ui::Layer::Ink2 pad=crate::components::ui::Pad::Md>
                    <p class="whitespace-pre-wrap break-words text-body text-primary" data-testid="reporter-words">{report.description.clone()}</p>
                </Surface>
            </div>
            <Field id="triage-status" label="Status">
                <Select value=status options=status_options />
            </Field>
            <Field id="triage-title" label="Title on the bug board" hint="Short and neutral, without names or team details. Up to 120 characters.">
                <TextField value=title maxlength=120 />
            </Field>
            <Field id="triage-note" label="Note to the reporter" hint="The reporter sees this. If the report is on the board, everyone signed in sees it too.">
                <TextField value=note rows=3 maxlength=600 />
            </Field>
            <Checkbox label="Show on the bug board" checked=on_board />
            {move || (status.get() == "fixed").then(|| view! {
                <Field id="triage-fixed" label="Fixed in" hint="The changelog entry that announced the fix. Only entries in the running version are listed.">
                    <Select value=fixed_in options=entry_options.get_value() />
                </Field>
            })}
            <Field id="triage-internal" label="Internal note" hint="Admins only.">
                <TextField value=internal rows=2 maxlength=2000 />
            </Field>
            {move || error.get().map(|e| view! { <p class="text-label text-danger" role="alert">{e}</p> })}
            <div class="flex flex-wrap items-center gap-3">
                <Button variant=ButtonVariant::Primary loading=saving on_click=Callback::new(save)>"Save"</Button>
                {is_wish.then(|| if has_proposal {
                    view! {
                        <Button variant=ButtonVariant::Secondary disabled=true>"Already a proposal"</Button>
                        <a class="text-label text-primary underline decoration-rule underline-offset-4" href=format!("/admin/proposals?id={}", proposal.clone().unwrap_or_default())>"Open the proposal"</a>
                    }.into_any()
                } else {
                    view! { <Button variant=ButtonVariant::Secondary on_click=Callback::new(make_proposal)>"Make a proposal"</Button> }.into_any()
                })}
            </div>
        </div>
    }
}
