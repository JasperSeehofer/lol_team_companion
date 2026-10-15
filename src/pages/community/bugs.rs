//! `/bugs`: the bug board and "Your reports" (signed-in beta users, across
//! teams). The board shows only what an admin wrote for everyone (a title
//! and a note); a reporter's own words appear only in their own tab.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{
    Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Callout, CalloutTone, Dialog, EmptySize,
    EmptyState, PageHeader, PageShell, PageWidth, Skeleton, SkeletonGroup, TabItem, TabPanel,
    Table, Tabs, Td, Th, Tr,
};
use crate::models::bug_report::{category_label, BugStatus, MyBugReport, PublicBugReport};
use crate::models::community::short_date;
use crate::pages::community::{redirect_signed_out, BugStatusBadge};

#[server]
pub async fn list_bug_board() -> Result<Vec<PublicBugReport>, ServerFnError> {
    use crate::server::{authz, community::store};
    let ctx = authz::authed().await?;
    store::list_bug_board(&ctx.db)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn list_my_bug_reports() -> Result<Vec<MyBugReport>, ServerFnError> {
    use crate::server::{authz, community::store};
    let ctx = authz::authed().await?;
    store::list_my_bug_reports(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn mark_my_reports_seen() -> Result<(), ServerFnError> {
    use crate::server::{authz, community::store};
    let ctx = authz::authed().await?;
    store::mark_my_reports_seen(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

const LEGEND: &str = "New: not looked at yet. Triaged: read and understood. Planned: on the list. Fixed: in a released update. Won't fix: the note says why.";

#[component]
pub fn BugBoardPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    redirect_signed_out(user);
    let board = Resource::new(|| (), |_| list_bug_board());
    let mine = Resource::new(|| (), |_| list_my_bug_reports());
    let query = leptos_router::hooks::use_query_map();
    let initial = if query.get_untracked().get("tab").as_deref() == Some("mine") {
        "mine"
    } else {
        "open"
    };
    let tab = RwSignal::new(initial.to_string());

    // Opening "Your reports" marks them seen once the list has rendered;
    // the "Updated" badges stay for this view (the list is not refetched).
    let marked = RwSignal::new(false);
    Effect::new(move |_| {
        if tab.get() == "mine" && matches!(mine.get(), Some(Ok(_))) && !marked.get_untracked() {
            marked.set(true);
            spawn_local(async move {
                let _ = mark_my_reports_seen().await;
            });
        }
    });

    view! {
        <PageShell width=PageWidth::Standard attr:data-feedback-label="Bug board">
            <PageHeader
                title="Bug board"
                eyebrow="Community"
                meta=|| view! { <span>"Problems and wishes reported by beta users, and what happened to them."</span> }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the bug board"><Skeleton lines=5 /></SkeletonGroup> }>
                {move || {
                    let (Some(board), Some(mine)) = (board.get(), mine.get()) else {
                        return None;
                    };
                    Some(match (board, mine) {
                        (Ok(board), Ok(mine)) => view! { <Board board=board mine=mine tab=tab /> }.into_any(),
                        _ => view! {
                            <Callout tone=CalloutTone::Danger>"This could not be loaded. Please try again in a moment."</Callout>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn Board(
    board: Vec<PublicBugReport>,
    mine: Vec<MyBugReport>,
    tab: RwSignal<String>,
) -> impl IntoView {
    let in_group = |r: &PublicBugReport, group: &str| {
        let s = BugStatus::parse(&r.status);
        match group {
            "open" => s.is_some_and(|s| s.is_open_on_board()),
            "fixed" => s == Some(BugStatus::Fixed),
            _ => s == Some(BugStatus::WontFix),
        }
    };
    let count = |g: &str| board.iter().filter(|r| in_group(r, g)).count() as u32;
    let updated = mine.iter().filter(|r| r.updated).count();
    let mine_label = if updated > 0 {
        format!("Your reports · {updated} updated")
    } else {
        "Your reports".to_string()
    };
    let tabs = vec![
        TabItem::new("open", "Open").with_count(count("open")),
        TabItem::new("fixed", "Fixed").with_count(count("fixed")),
        TabItem::new("wontfix", "Won't fix").with_count(count("wontfix")),
        TabItem::new("mine", mine_label).with_count(mine.len() as u32),
    ];
    let group_rows = |g: &'static str| -> Vec<PublicBugReport> {
        board.iter().filter(|r| in_group(r, g)).cloned().collect()
    };
    let (open, fixed, wontfix) = (
        group_rows("open"),
        group_rows("fixed"),
        group_rows("wontfix"),
    );
    view! {
        <Tabs group="bugs" label="Bug board" tabs=tabs selected=tab />
        <p class="mt-3 text-caption text-muted">{LEGEND}</p>
        <TabPanel group="bugs" tab="open" selected=tab>
            <BoardTable rows=open empty="No open reports on the board. When the team confirms a reported problem or wish, it appears here with its status." />
        </TabPanel>
        <TabPanel group="bugs" tab="fixed" selected=tab>
            <BoardTable rows=fixed empty="Nothing marked fixed yet. Fixed reports appear here with the update that fixed them." />
        </TabPanel>
        <TabPanel group="bugs" tab="wontfix" selected=tab>
            <BoardTable rows=wontfix empty="No reports were declined. If one is, it appears here with the reason." />
        </TabPanel>
        <TabPanel group="bugs" tab="mine" selected=tab>
            <MyReports rows=mine />
        </TabPanel>
        <p class="mt-8 text-caption text-muted">"Reports appear here after the team has read them, with a title the team writes. Your own words stay private."</p>
    }
}

#[component]
fn BoardTable(rows: Vec<PublicBugReport>, empty: &'static str) -> impl IntoView {
    if rows.is_empty() {
        return view! { <div class="mt-6"><EmptyState size=EmptySize::Compact message=empty /></div> }.into_any();
    }
    view! {
        <div class="mt-6">
            <Table caption="Reported problems and wishes" caption_hidden=true>
                <thead>
                    <tr>
                        <Th>"Report"</Th>
                        <Th>"Type"</Th>
                        <Th>"Status"</Th>
                        <Th>"Updated"</Th>
                        <Th>"Fixed in"</Th>
                    </tr>
                </thead>
                <tbody>
                    {rows.into_iter().map(|r| view! {
                        <Tr>
                            <Td>
                                <span class="text-body text-primary">{r.public_title.clone()}</span>
                                {r.public_note.clone().map(|n| view! { <span class="mt-1 block whitespace-pre-wrap text-caption text-secondary">{n}</span> })}
                            </Td>
                            <Td muted=true>{category_label(&r.category)}</Td>
                            <Td><BugStatusBadge status=r.status.clone() /></Td>
                            <Td muted=true>{r.status_changed_at.as_deref().map(short_date).unwrap_or_else(|| "\u{2014}".into())}</Td>
                            <Td>
                                {match (r.fixed_in.clone(), r.fixed_in_title.clone()) {
                                    (Some(slug), Some(title)) => view! {
                                        <a href=format!("/changelog#{slug}") class="text-primary underline decoration-rule underline-offset-4 hover:decoration-accent">{title}</a>
                                    }.into_any(),
                                    _ => view! { <span class="text-muted">"\u{2014}"</span> }.into_any(),
                                }}
                            </Td>
                        </Tr>
                    }).collect_view()}
                </tbody>
            </Table>
        </div>
    }
    .into_any()
}

#[component]
fn MyReports(rows: Vec<MyBugReport>) -> impl IntoView {
    if rows.is_empty() {
        return view! {
            <div class="mt-6">
                <EmptyState size=EmptySize::Compact message="You have not sent any feedback yet. Use Give Feedback at the bottom of any page; your reports and their status appear here." />
            </div>
        }
        .into_any();
    }
    let full_open = RwSignal::new(false);
    let full: RwSignal<(String, String)> = RwSignal::new((String::new(), String::new()));
    view! {
        <div class="mt-6">
            <Table caption="Reports you sent" caption_hidden=true>
                <thead>
                    <tr>
                        <Th>"Sent"</Th>
                        <Th>"Page"</Th>
                        <Th>"Your report"</Th>
                        <Th>"Status"</Th>
                        <Th>"Note from the team"</Th>
                    </tr>
                </thead>
                <tbody>
                    {rows.into_iter().map(|r| {
                        let short: String = r.description.chars().take(160).collect();
                        let long = r.description.chars().count() > 160;
                        let sent = short_date(&r.created_at);
                        let (title, body) = (format!("Your report from {sent}"), r.description.clone());
                        view! {
                            <Tr>
                                <Td muted=true>{sent.clone()}</Td>
                                <Td><code class="font-mono text-caption text-secondary">{r.page_url.clone()}</code></Td>
                                <Td>
                                    <span class="whitespace-pre-wrap text-body text-primary">{short}{long.then_some("\u{2026}")}</span>
                                    {long.then(|| view! {
                                        <span class="ml-2">
                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| {
                                                full.set((title.clone(), body.clone()));
                                                full_open.set(true);
                                            })>"Show all"</Button>
                                        </span>
                                    })}
                                </Td>
                                <Td>
                                    <span class="flex flex-wrap items-center gap-2">
                                        <BugStatusBadge status=r.status.clone() />
                                        {r.updated.then(|| view! { <Badge tone=BadgeTone::Accent>"Updated"</Badge> })}
                                    </span>
                                </Td>
                                <Td>
                                    <span class="whitespace-pre-wrap text-body text-secondary">{r.public_note.clone().unwrap_or_else(|| "\u{2014}".into())}</span>
                                    {r.fixed_in_title.clone().zip(r.fixed_in.clone()).map(|(t, slug)| view! {
                                        <a href=format!("/changelog#{slug}") class="mt-1 block text-caption text-primary underline decoration-rule underline-offset-4">{format!("Fixed in {t}")}</a>
                                    })}
                                </Td>
                            </Tr>
                        }
                    }).collect_view()}
                </tbody>
            </Table>
            <FullText open=full_open full=full />
        </div>
    }
    .into_any()
}

#[component]
fn FullText(open: RwSignal<bool>, full: RwSignal<(String, String)>) -> impl IntoView {
    // The dialog's title is fixed at creation; the date goes in the body.
    view! {
        <Dialog id="my-report-full" open=open title="Your report">
            <p class="text-caption text-muted">{move || full.get().0}</p>
            <p class="mt-3 whitespace-pre-wrap text-body text-primary">{move || full.get().1}</p>
        </Dialog>
    }
}
