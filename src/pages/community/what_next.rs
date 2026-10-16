//! `/what-next`: ideas the maintainer is considering, planning, building and
//! has shipped, with one vote per user per idea (signed-in beta users).
//! Only proposals an admin published appear; votes are counts, never names.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, EmptySize, EmptyState, PageHeader,
    PageShell, PageWidth, Skeleton, SkeletonGroup, TabItem, TabPanel, Tabs, ToastContext,
    ToastKind,
};
use crate::models::community::{votes_label, ProposalStatus, ProposalView};
use crate::pages::community::{redirect_signed_out, ProposalStatusBadge};

#[server]
pub async fn list_proposals() -> Result<Vec<ProposalView>, ServerFnError> {
    use crate::server::{authz, community::store};
    let ctx = authz::authed().await?;
    store::list_proposals(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

/// Vote (`true`) or take the vote back. Returns the new count.
#[server]
pub async fn set_vote(proposal_id: String, vote: bool) -> Result<u32, ServerFnError> {
    use crate::server::{authz, community::store, rate_limit::Bucket};
    let ctx = authz::authed().await?;
    authz::rate_limit_user(Bucket::Community, &ctx.user)?;
    let votes = store::set_vote(&ctx.db, &ctx.user.id, &proposal_id, vote)
        .await
        .map_err(authz::server_err)?;
    crate::server::bug_report_export::request_export();
    Ok(votes)
}

#[component]
pub fn WhatNextPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    redirect_signed_out(user);
    let proposals = Resource::new(|| (), |_| list_proposals());
    let tab = RwSignal::new("open".to_string());
    view! {
        <PageShell width=PageWidth::Reading attr:data-feedback-label="What next">
            <PageHeader
                title="What next"
                eyebrow="Community"
                meta=|| view! { <span>"Vote for what Team Companion should build. One vote per idea, and you can take it back."</span> }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading ideas"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || proposals.get().map(|r| match r {
                    Ok(list) => view! { <Board list=list tab=tab /> }.into_any(),
                    Err(_) => view! {
                        <Callout tone=CalloutTone::Danger>"This could not be loaded. Please try again in a moment."</Callout>
                    }.into_any(),
                })}
            </Suspense>
            <p class="mt-10 text-caption text-muted">"Missing an idea? Send it with Give Feedback and choose Idea."</p>
        </PageShell>
    }
}

#[component]
fn Board(list: Vec<ProposalView>, tab: RwSignal<String>) -> impl IntoView {
    let is = |p: &ProposalView, s: ProposalStatus| ProposalStatus::parse(&p.status) == Some(s);
    let open: Vec<ProposalView> = list
        .iter()
        .filter(|p| ProposalStatus::parse(&p.status).is_some_and(|s| s.is_open()))
        .cloned()
        .collect();
    let shipped: Vec<ProposalView> = list
        .iter()
        .filter(|p| is(p, ProposalStatus::Shipped))
        .cloned()
        .collect();
    let declined: Vec<ProposalView> = list
        .iter()
        .filter(|p| is(p, ProposalStatus::Declined))
        .cloned()
        .collect();
    let tabs = vec![
        TabItem::new("open", "Open").with_count(open.len() as u32),
        TabItem::new("shipped", "Shipped").with_count(shipped.len() as u32),
        TabItem::new("declined", "Declined").with_count(declined.len() as u32),
    ];
    view! {
        <Tabs group="what-next" label="Ideas" tabs=tabs selected=tab />
        <TabPanel group="what-next" tab="open" selected=tab>
            {if open.is_empty() {
                view! { <div class="mt-6"><EmptyState size=EmptySize::Compact message="No open ideas right now. New ideas appear here when the maintainer adds them, often from wishlist reports." /></div> }.into_any()
            } else {
                view! {
                    <ol class="mt-4" aria-label="Ideas, most votes first">
                        {open.into_iter().enumerate().map(|(i, p)| view! { <OpenIdea rank=i + 1 p=p /> }).collect_view()}
                    </ol>
                }.into_any()
            }}
        </TabPanel>
        <TabPanel group="what-next" tab="shipped" selected=tab>
            {if shipped.is_empty() {
                view! { <div class="mt-6"><EmptyState size=EmptySize::Compact message="Nothing shipped from this board yet. Ideas move here with a link to the update that delivered them." /></div> }.into_any()
            } else {
                view! {
                    <ul class="mt-4">
                        {shipped.into_iter().map(|p| view! { <ClosedIdea p=p /> }).collect_view()}
                    </ul>
                }.into_any()
            }}
        </TabPanel>
        <TabPanel group="what-next" tab="declined" selected=tab>
            {if declined.is_empty() {
                view! { <div class="mt-6"><EmptyState size=EmptySize::Compact message="No ideas were declined. If one is, it appears here with the reason." /></div> }.into_any()
            } else {
                view! {
                    <ul class="mt-4">
                        {declined.into_iter().map(|p| view! { <ClosedIdea p=p /> }).collect_view()}
                    </ul>
                }.into_any()
            }}
        </TabPanel>
    }
}

#[component]
fn Summary(text: String) -> impl IntoView {
    let long = text.chars().count() > 240;
    let expanded = RwSignal::new(false);
    let short: String = text.chars().take(240).collect();
    let text = StoredValue::new(text);
    view! {
        <p class="mt-1 max-w-[65ch] whitespace-pre-wrap text-body text-secondary">
            {move || if expanded.get() || !long { text.get_value() } else { format!("{short}\u{2026}") }}
        </p>
        {long.then(|| view! {
            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| expanded.update(|e| *e = !*e))>
                {move || if expanded.get() { "Less" } else { "More" }}
            </Button>
        })}
    }
}

#[component]
fn OpenIdea(rank: usize, p: ProposalView) -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let votes = RwSignal::new(p.votes);
    let voted = RwSignal::new(p.voted_by_me);
    let busy = RwSignal::new(false);
    let id = StoredValue::new(p.id.clone());
    let title = StoredValue::new(p.title.clone());
    let toggle = move |_| {
        if busy.get_untracked() {
            return;
        }
        let want = !voted.get_untracked();
        let before = (votes.get_untracked(), voted.get_untracked());
        // Optimistic, rolled back on failure.
        voted.set(want);
        votes.update(|v| *v = if want { *v + 1 } else { v.saturating_sub(1) });
        busy.set(true);
        spawn_local(async move {
            let result = set_vote(id.get_value(), want).await;
            busy.set(false);
            match result {
                Ok(n) => votes.set(n),
                Err(e) => {
                    votes.set(before.0);
                    voted.set(before.1);
                    let msg = if e.to_string().contains("closed") {
                        "Voting is closed for this idea."
                    } else {
                        "Your vote could not be saved. Please try again."
                    };
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Error, msg.into()));
                    }
                }
            }
        });
    };
    view! {
        <li class="grid grid-cols-[2rem_minmax(0,1fr)] gap-x-3 gap-y-3 border-b border-rule py-5 sm:grid-cols-[2rem_minmax(0,1fr)_auto]" data-proposal=p.id.clone()>
            <span class="figures pt-1 text-label text-muted">{rank}</span>
            <div class="min-w-0">
                <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                    <h2 class="font-display text-heading font-semibold text-primary">{p.title.clone()}</h2>
                    <ProposalStatusBadge status=p.status.clone() />
                </div>
                <Summary text=p.summary.clone() />
                {p.agent_proposed_on.clone().map(|d| view! { <p class="text-caption text-muted">{crate::models::community::agent_attribution(&d)}</p> })}
            </div>
            <div class="col-start-2 flex items-center gap-3 sm:col-start-3 sm:flex-col sm:items-end">
                <span class="figures text-label text-secondary" data-testid="vote-count">{move || votes_label(votes.get())}</span>
                <Button
                    size=ButtonSize::Sm
                    variant=ButtonVariant::Secondary
                    pressed=Signal::derive(move || voted.get())
                    aria_label=format!("Vote for {}", title.get_value())
                    on_click=Callback::new(toggle)
                >
                    {move || if voted.get() { "Voted" } else { "Vote" }}
                </Button>
            </div>
        </li>
    }
}

#[component]
fn ClosedIdea(p: ProposalView) -> impl IntoView {
    let shipped = ProposalStatus::parse(&p.status) == Some(ProposalStatus::Shipped);
    view! {
        <li class="border-b border-rule py-5">
            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                <h2 class="font-display text-heading font-semibold text-primary">{p.title.clone()}</h2>
                <ProposalStatusBadge status=p.status.clone() />
            </div>
            <Summary text=p.summary.clone() />
            {p.agent_proposed_on.clone().map(|d| view! { <p class="text-caption text-muted">{crate::models::community::agent_attribution(&d)}</p> })}
            {if shipped {
                view! {
                    <p class="mt-2 text-label text-secondary">
                        {p.shipped_in.clone().zip(p.shipped_in_title.clone()).map(|(slug, t)| view! {
                            "Shipped in "
                            <a href=format!("/changelog#{slug}") class="text-primary underline decoration-rule underline-offset-4 hover:decoration-accent">{t}</a>
                            ". "
                        })}
                        <span class="figures">{match p.votes { 1 => "1 person asked for this.".to_string(), n => format!("{n} people asked for this.") }}</span>
                    </p>
                }.into_any()
            } else {
                view! {
                    <p class="mt-2 text-label text-secondary">"Why not: "{p.status_note.clone().unwrap_or_default()}</p>
                }.into_any()
            }}
        </li>
    }
}
