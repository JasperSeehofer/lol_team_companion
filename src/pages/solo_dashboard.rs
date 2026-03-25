use crate::components::ui::{EmptyState, ErrorBanner, SkeletonCard, ToastContext, ToastKind};
use crate::models::match_data::PlayerMatchStats;
use crate::models::user::RankedInfo;
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server function return types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoloDashboardData {
    pub ranked: Option<RankedInfo>,
    pub matches: Vec<PlayerMatchStats>,
    pub should_auto_sync: bool,
    pub has_puuid: bool,
}

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_solo_dashboard(
    queue_filter: Option<i32>,
) -> Result<SoloDashboardData, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let has_puuid = user
        .riot_puuid
        .as_deref()
        .map(|p| !p.is_empty())
        .unwrap_or(false);

    // Read last_solo_sync from DB
    let should_auto_sync = db::get_should_auto_sync(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let ranked = db::get_latest_ranked_snapshot(&surreal, &user.id, "RANKED_SOLO_5x5")
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let matches = db::get_solo_matches(&surreal, &user.id, queue_filter, 20)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(SoloDashboardData {
        ranked,
        matches,
        should_auto_sync,
        has_puuid,
    })
}

#[server]
pub async fn sync_solo_matches() -> Result<i32, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let puuid = user
        .riot_puuid
        .as_deref()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| {
            ServerFnError::new("Link your Riot account on your profile first")
        })?
        .to_string();

    let region = user.riot_region.as_deref().unwrap_or("EUW").to_string();
    let platform = riot::platform_route_from_str(&region);

    let matches = riot::fetch_match_history(&puuid, None, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let new_count = matches.len();

    db::store_matches_with_synced_by(&surreal, &user.id, matches, Some(&user.id))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Fetch and store ranked snapshot
    let ranked_entries = riot::fetch_ranked_data(&puuid, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    for entry in ranked_entries {
        if entry.queue_type.contains("RANKED_SOLO") || entry.queue_type.contains("RANKED_FLEX") {
            db::store_ranked_snapshot(
                &surreal,
                &user.id,
                &entry.queue_type,
                &entry.tier,
                &entry.division,
                entry.lp,
                entry.wins,
                entry.losses,
            )
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        }
    }

    db::update_last_solo_sync(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(new_count as i32)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tier_emblem_url(tier: &str) -> String {
    format!(
        "https://ddragon.leagueoflegends.com/cdn/img/ranked-emblems/{}.png",
        tier.to_uppercase()
    )
}


// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn SoloDashboardPage() -> impl IntoView {
    // Auth redirect
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let toast = use_context::<ToastContext>().expect("ToastProvider");

    let queue_filter: RwSignal<Option<i32>> = RwSignal::new(None);
    let dashboard_resource = Resource::new(move || queue_filter.get(), |qf| get_solo_dashboard(qf));

    let syncing: RwSignal<bool> = RwSignal::new(false);
    let auto_synced: RwSignal<bool> = RwSignal::new(false);

    // Auto-sync on mount if stale
    Effect::new(move || {
        if let Some(Ok(data)) = dashboard_resource.get() {
            if data.should_auto_sync && data.has_puuid && !auto_synced.get_untracked() {
                auto_synced.set(true);
                syncing.set(true);
                leptos::task::spawn_local(async move {
                    match sync_solo_matches().await {
                        Ok(n) => {
                            if n > 0 {
                                toast
                                    .show
                                    .run((ToastKind::Success, format!("Synced {n} new matches")));
                            } else {
                                toast
                                    .show
                                    .run((ToastKind::Success, "Already up to date".to_string()));
                            }
                            dashboard_resource.refetch();
                        }
                        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                    }
                    syncing.set(false);
                });
            }
        }
    });

    let do_sync = move |_| {
        if syncing.get_untracked() {
            return;
        }
        syncing.set(true);
        leptos::task::spawn_local(async move {
            match sync_solo_matches().await {
                Ok(n) => {
                    if n > 0 {
                        toast
                            .show
                            .run((ToastKind::Success, format!("Synced {n} new matches")));
                    } else {
                        toast
                            .show
                            .run((ToastKind::Success, "Already up to date".to_string()));
                    }
                    dashboard_resource.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
            syncing.set(false);
        });
    };

    view! {
        <div class="max-w-2xl mx-auto py-8 px-6 flex flex-col gap-8">

            // ── Header + Sync Button ────────────────────────────────────────
            <div class="flex items-center justify-between">
                <h1 class="text-3xl font-bold text-primary">"My Dashboard"</h1>
                <button
                    class=move || if syncing.get() {
                        "bg-accent opacity-60 cursor-not-allowed text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm"
                    } else {
                        "bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors"
                    }
                    on:click=do_sync
                    disabled=move || syncing.get()
                >
                    {move || if syncing.get() {
                        view! { <span>"Syncing..."</span> }.into_any()
                    } else {
                        view! { <span>"Sync Matches"</span> }.into_any()
                    }}
                </button>
            </div>

            // ── Ranked Badge + Matches + Goals ──────────────────────────────
            <Suspense fallback=|| view! {
                <div class="flex flex-col gap-4">
                    <SkeletonCard height="h-28" />
                    <SkeletonCard height="h-16" />
                    <SkeletonCard height="h-16" />
                    <SkeletonCard height="h-16" />
                </div>
            }>
                {move || dashboard_resource.get().map(|result| match result {
                    Err(e) => view! {
                        <ErrorBanner message=format!("Failed to load dashboard: {e}") />
                    }.into_any(),
                    Ok(data) => view! {
                        <div class="flex flex-col gap-8">
                            <RankedBadgeSection ranked=data.ranked />
                            <MatchListSection
                                matches=data.matches
                                queue_filter=queue_filter
                            />
                            <GoalPlaceholders />
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

#[component]
fn RankedBadgeSection(ranked: Option<RankedInfo>) -> impl IntoView {
    view! {
        <div class="bg-elevated border border-divider rounded-xl p-6">
            <p class="text-xs text-dimmed uppercase tracking-wider mb-3">"Ranked Solo/Duo"</p>
            {match ranked {
                Some(info) => {
                    let tier = info.tier.clone();
                    let tier_for_img = info.tier.clone();
                    let division = info.division.clone();
                    let display_name = if division.is_empty() {
                        info.tier.clone()
                    } else {
                        format!("{} {}", info.tier, info.division)
                    };
                    let total = info.wins + info.losses;
                    let wr = if total > 0 {
                        format!("{}%", (info.wins as f64 / total as f64 * 100.0).round() as i32)
                    } else {
                        "0%".to_string()
                    };

                    view! {
                        <div class="flex items-center gap-4">
                            <img
                                src=tier_emblem_url(&tier_for_img)
                                alt=tier.clone()
                                class="w-16 h-16 object-contain"
                            />
                            <div class="flex flex-col gap-1">
                                <span class="text-3xl font-semibold text-primary">{display_name}</span>
                                <span class="text-xl font-semibold text-secondary">{format!("{} LP", info.lp)}</span>
                                <span class="text-sm text-muted">
                                    {format!("{}W {}L ({})", info.wins, info.losses, wr)}
                                </span>
                            </div>
                        </div>
                    }.into_any()
                }
                None => view! {
                    <div class="flex items-center gap-4">
                        <svg
                            class="w-16 h-16 text-dimmed"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                            stroke-width="1.5"
                        >
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
                        </svg>
                        <span class="text-xl font-semibold text-muted">"Unranked"</span>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

#[component]
fn MatchListSection(
    matches: Vec<PlayerMatchStats>,
    queue_filter: RwSignal<Option<i32>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            // Section header
            <div class="flex items-center justify-between">
                <h2 class="text-xl font-semibold text-primary">"Recent Matches"</h2>
                <select
                    class="bg-surface border border-outline/50 rounded-lg px-3 py-2 text-sm text-secondary"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        let qf = match val.as_str() {
                            "420" => Some(420_i32),
                            "440" => Some(440_i32),
                            _ => None,
                        };
                        queue_filter.set(qf);
                    }
                >
                    <option value="">"All Queues"</option>
                    <option value="420">"Solo/Duo"</option>
                    <option value="440">"Flex"</option>
                </select>
            </div>

            // Match list
            {if matches.is_empty() {
                view! {
                    <EmptyState
                        message="No matches yet — sync your match history to see recent games here."
                    />
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col gap-2">
                        {matches.into_iter().map(|m| {
                            let border_class = if m.win {
                                "border-l-4 border-blue-500"
                            } else {
                                "border-l-4 border-red-500/50"
                            };
                            let row_class = format!(
                                "bg-surface {} rounded-lg p-3 flex items-center gap-3",
                                border_class
                            );
                            let kda = format!("{}/{}/{}", m.kills, m.deaths, m.assists);
                            let cs_str = format!("{} CS", m.cs);

                            view! {
                                <div class=row_class>
                                    <span class="text-sm font-medium text-primary flex-1">{m.champion}</span>
                                    <span class="text-sm text-secondary">{kda}</span>
                                    <span class="text-xs text-muted">{cs_str}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn GoalPlaceholders() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <h2 class="text-xl font-semibold text-primary">"Goals"</h2>
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                // Rank Target
                <div class="bg-elevated border border-divider/50 rounded-xl p-4 opacity-60 cursor-default">
                    <div class="flex flex-col gap-1">
                        <svg class="w-5 h-5 text-dimmed mb-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                        </svg>
                        <span class="text-secondary text-sm font-medium">"Rank Target"</span>
                        <span class="text-dimmed text-xs">"Coming in a future update"</span>
                    </div>
                </div>
                // CS per Minute
                <div class="bg-elevated border border-divider/50 rounded-xl p-4 opacity-60 cursor-default">
                    <div class="flex flex-col gap-1">
                        <svg class="w-5 h-5 text-dimmed mb-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 18L9 11.25l4.306 4.307a11.95 11.95 0 015.814-5.519l2.74-1.22m0 0l-5.94-2.28m5.94 2.28l-2.28 5.941" />
                        </svg>
                        <span class="text-secondary text-sm font-medium">"CS per Minute"</span>
                        <span class="text-dimmed text-xs">"Coming in a future update"</span>
                    </div>
                </div>
                // Deaths per Game
                <div class="bg-elevated border border-divider/50 rounded-xl p-4 opacity-60 cursor-default">
                    <div class="flex flex-col gap-1">
                        <svg class="w-5 h-5 text-dimmed mb-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                        </svg>
                        <span class="text-secondary text-sm font-medium">"Deaths per Game"</span>
                        <span class="text-dimmed text-xs">"Coming in a future update"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
