//! `/admin/reimport`: re-import stored games with the current timeline
//! importer (M149 item A2). Games cached before M149 have no event
//! positions and no item purchases, so their timelines show no inferred
//! recalls. One batch re-fetches up to [`REIMPORT_BATCH`] of them from Riot
//! through the shared client (global request budget) and replaces each
//! game's timeline in one transaction; it is idempotent and resumable (the
//! importer version on `match_detail` says what is still old).
//!
//! Admin-only like `/admin/resets`: every server function checks
//! `require_admin()`, everyone else gets the 404 page (`SsrMode::Async`).

use leptos::prelude::*;

use crate::components::ui::{
    Button, ButtonSize, ButtonVariant, Callout, CalloutTone, EmptySize, EmptyState, PageHeader,
    PageShell, PageWidth, SectionHeader, Skeleton, SkeletonGroup,
};
use crate::error_template::NotFound;
use crate::models::match_data::ReimportReport;

/// Games one batch re-imports (two Riot requests each).
pub const REIMPORT_BATCH: usize = 5;

/// (waiting, unavailable, Riot configured).
#[server]
pub async fn reimport_status() -> Result<(u32, u32, bool), ServerFnError> {
    use crate::server::{authz, db};
    authz::require_admin().await?;
    let db = authz::db()?;
    let (waiting, unavailable) = db::reimport_counts(&db).await.map_err(authz::server_err)?;
    let configured = authz::riot()?.is_configured();
    Ok((waiting, unavailable, configured))
}

/// Re-import the next batch of stored games. Match ids come from the
/// database only (never from the client); a game Riot no longer has is
/// marked and skipped from then on, while a busy or failing Riot API stops
/// the batch without marking anything.
#[server]
pub async fn reimport_batch() -> Result<ReimportReport, ServerFnError> {
    use crate::server::{authz, db, rate_limit::Bucket, riot};

    let admin = authz::require_admin().await?;
    authz::rate_limit_user(Bucket::Reimport, &admin)?;
    let db = authz::db()?;
    let client = authz::riot()?;
    if !client.is_configured() {
        return Err(ServerFnError::new(
            "Riot features are not set up on this server (no RIOT_API_KEY), so stored games cannot be re-imported.",
        ));
    }
    let ids = db::matches_needing_reimport(&db, REIMPORT_BATCH)
        .await
        .map_err(authz::server_err)?;
    let mut report = ReimportReport::default();
    for match_id in ids {
        if let Err(e) = client.ensure_headroom(2) {
            report.stopped = Some(error_message(authz::server_err(e)));
            break;
        }
        let Some(platform) = riot::platform_from_match_id(&match_id) else {
            db::mark_reimport_failed(&db, &match_id)
                .await
                .map_err(authz::server_err)?;
            report.failed += 1;
            continue;
        };
        match client.fetch_match_and_timeline(&match_id, platform).await {
            Ok((m, Some(timeline))) => {
                let full = riot::full_match_from_riot(&match_id, &m, Some(&timeline));
                match db::replace_match_timeline(
                    &db,
                    &match_id,
                    &full.participants,
                    &full.raw_timeline_events,
                    full.game_duration,
                    &full.game_mode,
                )
                .await
                {
                    Ok(()) => report.done += 1,
                    // Purged by the retention task meanwhile: nothing to do.
                    Err(db::DbError::NotFound) => {}
                    Err(e) => return Err(authz::server_err(e)),
                }
            }
            // No timeline: keep the stored one rather than replace it with nothing.
            Ok((_, None)) => {
                db::mark_reimport_failed(&db, &match_id)
                    .await
                    .map_err(authz::server_err)?;
                report.failed += 1;
            }
            Err(e) if e.is_permanent() => {
                tracing::info!(%match_id, error = %e, "re-import: Riot has no data for this game");
                db::mark_reimport_failed(&db, &match_id)
                    .await
                    .map_err(authz::server_err)?;
                report.failed += 1;
            }
            Err(e) => {
                report.stopped = Some(error_message(authz::server_err(e)));
                break;
            }
        }
    }
    let (waiting, unavailable) = db::reimport_counts(&db).await.map_err(authz::server_err)?;
    report.remaining = waiting;
    report.unavailable = unavailable;
    tracing::info!(
        admin = %admin.id,
        done = report.done,
        failed = report.failed,
        remaining = report.remaining,
        "stored games re-imported"
    );
    Ok(report)
}

#[cfg(feature = "ssr")]
fn error_message(e: ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(m) => m,
        other => other.to_string(),
    }
}

fn games(n: u32) -> String {
    if n == 1 {
        "1 game".to_string()
    } else {
        format!("{n} games")
    }
}

/// The plain-words result of one batch.
pub fn report_text(r: &ReimportReport) -> String {
    let mut text = format!("Re-imported {}.", games(r.done));
    if r.failed > 0 {
        text.push_str(&format!(
            " Riot has no data for {}; they keep their old timeline.",
            games(r.failed)
        ));
    }
    if r.remaining == 0 {
        text.push_str(" Every stored game is up to date.");
    } else {
        text.push_str(&format!(" {} still to go.", games(r.remaining)));
    }
    text
}

#[component]
pub fn AdminReimportPage() -> impl IntoView {
    let status = Resource::new(|| (), |_| reimport_status());
    let run = ServerAction::<ReimportBatch>::new();

    Effect::new(move |_| {
        if run.value().get().is_some() {
            status.refetch();
        }
    });
    let result = move || {
        run.value().get().map(|r| match r {
            Ok(report) => {
                let text = report_text(&report);
                let stopped = report.stopped.clone();
                let tone = if stopped.is_some() {
                    CalloutTone::Warning
                } else {
                    CalloutTone::Success
                };
                view! {
                    <Callout tone=tone title="Batch finished">
                        <p>{text}</p>
                        {stopped.map(|s| view! { <p class="mt-1">{format!("Stopped early: {s}")}</p> })}
                    </Callout>
                }
                .into_any()
            }
            Err(e) => view! {
                <Callout tone=CalloutTone::Danger title="Re-import failed">
                    <p>{crate::models::utils::error_text(&e)}</p>
                </Callout>
            }
            .into_any(),
        })
    };

    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading re-import"><Skeleton lines=4 /></SkeletonGroup></PageShell> }>
            {move || status.get().map(|r| match r {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((waiting, unavailable, configured)) => view! {
                    <PageShell width=PageWidth::Standard attr:data-feedback-label="Re-import games">
                        <PageHeader
                            title="Re-import games"
                            eyebrow="Admin"
                            meta=move || view! { <span>"Games opened before the timeline kept positions and item purchases show no recalls. Re-importing fetches them from Riot again."</span> }
                        />
                        <section aria-labelledby="reimport-h">
                            <SectionHeader title="Stored games" id="reimport-h" />
                            {(!configured).then(|| view! {
                                <div class="mb-4">
                                    <Callout tone=CalloutTone::Warning title="Riot is not set up">
                                        <p>"This server has no RIOT_API_KEY, so stored games cannot be re-imported."</p>
                                    </Callout>
                                </div>
                            })}
                            {if waiting == 0 {
                                view! { <EmptyState size=EmptySize::Compact message="Every stored game is up to date. Games opened from now on are imported with positions and purchases." /> }.into_any()
                            } else {
                                view! {
                                    <p class="text-body text-secondary" data-testid="reimport-waiting">
                                        {format!("{} waiting. Each batch re-imports up to {REIMPORT_BATCH}, two Riot requests each.", games(waiting))}
                                    </p>
                                    <div class="mt-3">
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Touch
                                            disabled=!configured
                                            loading=run.pending()
                                            on_click=Callback::new(move |_| { run.dispatch(ReimportBatch {}); })
                                        >
                                            {format!("Re-import the next {REIMPORT_BATCH}")}
                                        </Button>
                                    </div>
                                }.into_any()
                            }}
                            {(unavailable > 0).then(|| view! {
                                <p class="mt-3 text-caption text-muted">
                                    {format!("Riot has no data for {} any more; they keep their old timeline and are not asked for again.", games(unavailable))}
                                </p>
                            })}
                            <div class="mt-4" role="status" data-testid="reimport-result">{result}</div>
                        </section>
                    </PageShell>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_says_what_happened() {
        let r = ReimportReport {
            done: 5,
            failed: 0,
            remaining: 12,
            unavailable: 0,
            stopped: None,
        };
        assert_eq!(
            report_text(&r),
            "Re-imported 5 games. 12 games still to go."
        );
        let r = ReimportReport {
            done: 1,
            failed: 1,
            remaining: 0,
            unavailable: 1,
            stopped: None,
        };
        assert_eq!(
            report_text(&r),
            "Re-imported 1 game. Riot has no data for 1 game; they keep their old timeline. Every stored game is up to date."
        );
    }
}
