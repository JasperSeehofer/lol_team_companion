//! `/changelog` (public): `CHANGELOG.md` of the running build, newest first,
//! with detected game patches beside it. Signed-in users also see which
//! bug-board reports an entry fixed and which What next ideas it shipped.

use leptos::prelude::*;

use crate::components::ui::{
    Callout, CalloutTone, EmptySize, EmptyState, PageHeader, PageShell, PageWidth, SectionHeader,
    Skeleton, SkeletonGroup,
};
use crate::models::community::{short_date, ChangelogView, Span, SpanKind};

#[server]
pub async fn get_changelog() -> Result<ChangelogView, ServerFnError> {
    use crate::models::community::PatchNote;
    use crate::server::{authz, community::changelog, community::store, db, rate_limit::Bucket};

    authz::rate_limit(Bucket::PublicRead).await?;
    let db = authz::db()?;
    let patches = db::list_patch_changes(&db, 30)
        .await
        .map_err(authz::server_err)?
        .into_iter()
        .map(|p| PatchNote {
            version: p.new_version,
            previous: p.old_version,
            detected_at: p.detected_at,
        })
        .collect();
    let links = if authz::optional_user().await?.is_some() {
        store::changelog_links(&db)
            .await
            .map_err(authz::server_err)?
    } else {
        Vec::new()
    };
    Ok(ChangelogView {
        entries: changelog::entries().to_vec(),
        patches,
        links,
    })
}

#[component]
pub fn ChangelogPage() -> impl IntoView {
    let data = Resource::new(|| (), |_| get_changelog());
    view! {
        <PageShell width=PageWidth::Standard>
            <PageHeader
                title="Changelog"
                eyebrow="Community"
                meta=|| view! { <span>"What changed in Companion, newest first."</span> }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the changelog"><Skeleton lines=8 /></SkeletonGroup> }>
                {move || data.get().map(|r| match r {
                    Ok(view) => view! { <Changelog data=view /> }.into_any(),
                    Err(_) => view! {
                        <Callout tone=CalloutTone::Danger>"This could not be loaded. Please try again in a moment."</Callout>
                    }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

fn spans(item: Vec<Span>) -> impl IntoView {
    item.into_iter()
        .map(|s| match s.kind {
            SpanKind::Code => {
                view! { <code class="font-mono text-primary">{s.text}</code> }.into_any()
            }
            SpanKind::Strong => {
                view! { <strong class="font-semibold text-primary">{s.text}</strong> }.into_any()
            }
            SpanKind::Text => s.text.into_any(),
        })
        .collect_view()
}

#[component]
fn Changelog(data: ChangelogView) -> impl IntoView {
    let ChangelogView {
        entries,
        patches,
        links,
    } = data;
    let link_class =
        "text-primary underline decoration-rule underline-offset-4 hover:decoration-accent";
    view! {
        <div class="grid gap-12 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
            <div class="min-w-0">
                {if entries.is_empty() {
                    view! { <EmptyState message="No changes recorded yet. Each update to Companion will be listed here." /> }.into_any()
                } else {
                    entries.into_iter().map(|e| {
                        let extras = links.iter().find(|l| l.slug == e.slug).cloned();
                        let meta = e.date.as_deref().map(short_date).unwrap_or_else(|| "Date not recorded".to_string());
                        view! {
                            <section class="scroll-mt-20 border-t border-rule pt-8 pb-10 first:border-t-0 first:pt-0" id=e.slug.clone() aria-labelledby=format!("{}-h", e.slug)>
                                <SectionHeader title=e.title.clone() meta=meta id=format!("{}-h", e.slug) />
                                {e.sections.into_iter().filter(|s| !s.items.is_empty()).map(|s| view! {
                                    <h3 class="mt-6 font-display text-heading font-semibold text-primary">{s.heading}</h3>
                                    <ul class="mt-2 flex max-w-[70ch] list-disc flex-col gap-2 pl-5 text-reading text-secondary">
                                        {s.items.into_iter().map(|i| view! { <li>{spans(i)}</li> }).collect_view()}
                                    </ul>
                                }).collect_view()}
                                {extras.map(|x| view! {
                                    {(!x.fixed_reports.is_empty()).then(|| view! {
                                        <h3 class="mt-6 font-display text-heading font-semibold text-primary">"Fixed from the bug board"</h3>
                                        <ul class="mt-2 flex flex-col gap-1 text-reading">
                                            {x.fixed_reports.into_iter().map(|(_, t)| view! { <li><a href="/bugs" class=link_class>{t}</a></li> }).collect_view()}
                                        </ul>
                                    })}
                                    {(!x.shipped_proposals.is_empty()).then(|| view! {
                                        <h3 class="mt-6 font-display text-heading font-semibold text-primary">"Shipped from What next"</h3>
                                        <ul class="mt-2 flex flex-col gap-1 text-reading">
                                            {x.shipped_proposals.into_iter().map(|(_, t)| view! { <li><a href="/what-next" class=link_class>{t}</a></li> }).collect_view()}
                                        </ul>
                                    })}
                                })}
                            </section>
                        }
                    }).collect_view().into_any()
                }}
            </div>
            <aside class="min-w-0" aria-labelledby="patches-h">
                <SectionHeader title="Game patches" id="patches-h" />
                {if patches.is_empty() {
                    view! { <EmptyState size=EmptySize::Compact message="No game patch detected yet. When a new patch is released, it appears here within about six hours." /> }.into_any()
                } else {
                    view! {
                        <ul class="flex flex-col">
                            {patches.into_iter().map(|p| view! {
                                <li class="border-b border-rule py-3">
                                    <p class="text-body text-primary">"Patch "<span class="figures">{p.version}</span>" detected on "{short_date(&p.detected_at)}</p>
                                    <p class="text-caption text-secondary">"Champion data and pictures now follow this patch."</p>
                                    {p.previous.map(|v| view! { <p class="text-caption text-muted">"Previous: "<span class="figures">{v}</span></p> })}
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </aside>
        </div>
    }
}
