//! `/transparency` (public): where support for Companion goes, month by
//! month. Every money sentence and label comes from an admin-approved site
//! text; until the owner approves them, the page says only "No months
//! published yet." and nothing about money. The support link appears only
//! with `SUPPORT_URL` set and its sentence approved.

use leptos::prelude::*;

use crate::components::ui::{
    BarInk, BarSegment, Button, ButtonVariant, Callout, CalloutTone, EmptyState, PageHeader,
    PageShell, PageWidth, ProportionBar, ReadoutSize, SectionHeader, Skeleton, SkeletonGroup,
    StatReadout, Table, Td, Th, Tr,
};
use crate::models::community::{format_euros, month_label, short_date, LedgerMonth, LedgerView};

/// The neutral state shown while the money wording is not approved.
pub const GATE_CLOSED: &str = "No months published yet.";

#[server]
pub async fn get_ledger() -> Result<LedgerView, ServerFnError> {
    use crate::server::{authz, rate_limit::Bucket};
    authz::rate_limit(Bucket::PublicRead).await?;
    let db = authz::db()?;
    build_ledger_view(&db).await.map_err(authz::server_err)
}

/// The public view from the database: only approved texts, and months only
/// with approved labels (SSR only).
#[cfg(feature = "ssr")]
pub async fn build_ledger_view(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
) -> Result<LedgerView, crate::server::db::DbError> {
    use crate::models::community::{
        ledger_totals, paragraphs, LedgerLabels, SupportLink, TEXT_LEDGER_LABELS,
        TEXT_MONEY_POLICY, TEXT_NOT_COVERED, TEXT_SPLIT_LEGEND, TEXT_SUPPORT_SENTENCE,
    };
    use crate::server::community::store;

    let texts = store::approved_texts(db).await?;
    let policy = texts.get(TEXT_MONEY_POLICY).map(|b| paragraphs(b));
    let labels = texts
        .get(TEXT_LEDGER_LABELS)
        .and_then(|b| LedgerLabels::parse(b));
    let mut view = LedgerView {
        policy: policy.clone(),
        labels: labels.clone(),
        split_legend: texts.get(TEXT_SPLIT_LEGEND).cloned(),
        not_covered_note: texts.get(TEXT_NOT_COVERED).cloned(),
        ..Default::default()
    };
    // Figures need the policy and the labels; the support link needs the
    // policy, its sentence and SUPPORT_URL.
    if policy.is_some() && labels.is_some() {
        view.months = store::list_published_ledger(db).await?;
        view.totals = ledger_totals(&view.months.iter().take(12).cloned().collect::<Vec<_>>());
    }
    if policy.is_some() {
        if let (Some(url), Some(sentence)) = (
            crate::server::config::get().support_url.clone(),
            texts.get(TEXT_SUPPORT_SENTENCE).cloned(),
        ) {
            view.support = Some(SupportLink { url, sentence });
        }
    }
    Ok(view)
}

#[component]
pub fn TransparencyPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| get_ledger());
    view! {
        <PageShell width=PageWidth::Standard>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the transparency page"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || data.get().map(|r| match r {
                    Ok(v) => view! { <Ledger data=v /> }.into_any(),
                    Err(_) => view! {
                        <PageHeader title="Transparency" eyebrow="Community" />
                        <Callout tone=CalloutTone::Danger>"This could not be loaded. Please try again in a moment."</Callout>
                    }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn Ledger(data: LedgerView) -> impl IntoView {
    let Some(policy) = data.policy.clone() else {
        return view! {
            <PageHeader title="Transparency" eyebrow="Community" />
            <EmptyState message=GATE_CLOSED />
        }
        .into_any();
    };
    let LedgerView {
        labels,
        split_legend,
        not_covered_note,
        months,
        totals,
        support,
        ..
    } = data;
    view! {
        <PageHeader
            title="Transparency"
            eyebrow="Community"
            meta=|| view! { <span>"Where support for Companion goes, month by month."</span> }
        />
        <section aria-labelledby="policy-h" class="max-w-[70ch]">
            <SectionHeader title="How support is used" id="policy-h" />
            <div class="flex flex-col gap-4 text-reading text-secondary" data-testid="money-policy">
                {policy.into_iter().map(|p| view! { <p>{p}</p> }).collect_view()}
            </div>
        </section>
        {match labels {
            Some(l) if !months.is_empty() => {
                let max = months.iter().map(|m| m.support_cents.max(m.costs_cents())).max().unwrap_or(0);
                view! {
                    <section aria-labelledby="totals-h" class="mt-12">
                        <SectionHeader title="Last 12 months" id="totals-h" />
                        <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
                            <StatReadout size=ReadoutSize::Md label=l.support.clone() value=format_euros(totals.support_cents) />
                            <StatReadout size=ReadoutSize::Md label=l.costs.clone() value=format_euros(totals.costs_cents) />
                            <StatReadout size=ReadoutSize::Md label=l.to_maintainer.clone() value=format_euros(totals.to_maintainer_cents) />
                            {(totals.not_covered_cents > 0).then(|| view! {
                                <StatReadout size=ReadoutSize::Md label=l.not_covered.clone() value=format_euros(totals.not_covered_cents) />
                            })}
                        </div>
                    </section>
                    <section aria-labelledby="months-h" class="mt-12">
                        <SectionHeader title="Month by month" id="months-h" />
                        {split_legend.map(|t| view! { <p class="mb-3 text-caption text-muted">{t}</p> })}
                        <MonthTable months=months labels=l max=max not_covered_note=not_covered_note />
                    </section>
                }.into_any()
            }
            _ => view! {
                <section class="mt-12"><EmptyState message=GATE_CLOSED /></section>
            }.into_any(),
        }}
        {support.map(|s| view! {
            <section aria-labelledby="support-h" class="mt-12 max-w-[70ch]">
                <SectionHeader title="Support" id="support-h" />
                <p class="text-reading text-secondary">{s.sentence}</p>
                <div class="mt-4">
                    <SupportButton url=s.url />
                </div>
            </section>
        })}
    }
    .into_any()
}

/// The one external link: opens the support page in a new tab, no referrer.
#[component]
fn SupportButton(url: String) -> impl IntoView {
    view! {
        <Button
            href=url
            variant=ButtonVariant::Secondary
            aria_label="Support Companion (opens an external site)"
            attr:target="_blank"
            attr:rel="noopener noreferrer external"
            attr:data-testid="support-link"
        >
            "Support Companion"
        </Button>
    }
}

#[component]
fn MonthTable(
    months: Vec<LedgerMonth>,
    labels: crate::models::community::LedgerLabels,
    max: i64,
    not_covered_note: Option<String>,
) -> impl IntoView {
    let show_all = RwSignal::new(false);
    let more = months.len() > 12;
    let rows = months
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            let costs = m.costs_cents();
            let short = m.not_covered_cents();
            let covered = m.support_cents.min(costs);
            let note = m.note.clone();
            let revised = m.revised_at.clone();
            let other_label = m.other_label.clone();
            let caption = note.is_some() || revised.is_some() || (short > 0 && not_covered_note.is_some()) || other_label.is_some();
            let ncn = not_covered_note.clone();
            // The approved label, not a sentence of our own (money wording).
            let not_covered_label = labels.not_covered.clone();
            let label = month_label(&m.month);
            let to_m = m.to_maintainer_cents();
            let (support, hosting, domain, ai, other) = (m.support_cents, m.hosting_cents, m.domain_cents, m.ai_tokens_cents, m.other_cents);
            view! {
                <tbody class=move || { if i >= 12 && !show_all.get() { "hidden" } else { "" } }>
                    <Tr>
                        <Td header=true>{label}</Td>
                        <Td numeric=true>{format_euros(support)}</Td>
                        <Td numeric=true>{format_euros(hosting)}</Td>
                        <Td numeric=true>{format_euros(domain)}</Td>
                        <Td numeric=true>{format_euros(ai)}</Td>
                        <Td numeric=true>{format_euros(other)}</Td>
                        <Td numeric=true>{format_euros(costs)}</Td>
                        <Td numeric=true>
                            {format_euros(to_m)}
                            {(short > 0).then(|| view! { <span class="block text-caption text-secondary">{format!("{}: {}", not_covered_label, format_euros(short))}</span> })}
                        </Td>
                        <Td>
                            <div class="w-28">
                                <ProportionBar max=max segments=vec![
                                    BarSegment::new(covered, "Costs covered", BarInk::Strong),
                                    BarSegment::new(to_m, "To the maintainer", BarInk::Soft),
                                    BarSegment::new(short, "Not covered", BarInk::Hollow),
                                ] />
                            </div>
                        </Td>
                    </Tr>
                    {caption.then(|| view! {
                        <tr>
                            <td colspan="9" class="px-3 pb-3 text-caption text-muted">
                                {other_label.map(|o| view! { <span class="mr-3">{format!("Other: {o}")}</span> })}
                                {(short > 0).then(|| ncn.map(|t| view! { <span class="mr-3">{t}</span> }))}
                                {note.map(|n| view! { <span class="mr-3">{n}</span> })}
                                {revised.map(|r| view! { <span>{format!("Revised on {}.", short_date(&r))}</span> })}
                            </td>
                        </tr>
                    })}
                </tbody>
            }
        })
        .collect_view();
    view! {
        <Table caption="Support and costs per month">
            <thead>
                <tr>
                    <Th>"Month"</Th>
                    <Th numeric=true>{labels.support.clone()}</Th>
                    <Th numeric=true>{labels.hosting.clone()}</Th>
                    <Th numeric=true>{labels.domain.clone()}</Th>
                    <Th numeric=true>{labels.ai_tokens.clone()}</Th>
                    <Th numeric=true>{labels.other.clone()}</Th>
                    <Th numeric=true>{labels.costs.clone()}</Th>
                    <Th numeric=true>{labels.to_maintainer.clone()}</Th>
                    <Th>"Split"</Th>
                </tr>
            </thead>
            {rows}
        </Table>
        {more.then(|| view! {
            <div class="mt-3">
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| show_all.set(true))>"Show earlier months"</Button>
            </div>
        })}
    }
}
