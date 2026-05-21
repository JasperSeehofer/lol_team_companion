use crate::app::InitialTheme;
use crate::components::region::*;
use crate::components::skeleton::{PageEmpty, PageLoading};
use crate::components::ui::{ErrorBanner, StatusMessage, ToastContext, ToastKind};
use crate::models::match_data::{GoalProgressPayload, PlayerMatchStats, RankedSnapshot};
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

#[server]
pub async fn get_lp_history(window: String) -> Result<Vec<RankedSnapshot>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};
    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    let cutoff = window_to_cutoff(&window);
    db::get_lp_history(&surreal, &user.id, cutoff).await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn compute_goal_progress() -> Result<GoalProgressPayload, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};
    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::compute_goal_progress(&surreal, &user.id).await.map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn upsert_personal_goal(goal_type: String, target_value: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};
    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;
    if !matches!(goal_type.as_str(), "rank_target" | "cs_per_min" | "deaths_per_game") {
        return Err(ServerFnError::new("Invalid goal_type"));
    }
    if goal_type == "cs_per_min" {
        let v: f32 = target_value.parse().map_err(|_| ServerFnError::new("CS/min must be a number"))?;
        if !(0.0..=15.0).contains(&v) { return Err(ServerFnError::new("CS/min must be between 0 and 15")); }
    }
    if goal_type == "deaths_per_game" {
        let v: i32 = target_value.parse().map_err(|_| ServerFnError::new("Deaths must be a whole number"))?;
        if !(0..=20).contains(&v) { return Err(ServerFnError::new("Deaths must be between 0 and 20")); }
    }
    if goal_type == "rank_target" {
        let parts: Vec<&str> = target_value.splitn(2, ':').collect();
        let tier = parts.first().copied().unwrap_or("");
        if !matches!(tier.to_uppercase().as_str(), "IRON" | "BRONZE" | "SILVER" | "GOLD" | "PLATINUM" | "EMERALD" | "DIAMOND" | "MASTER" | "GRANDMASTER" | "CHALLENGER") {
            return Err(ServerFnError::new("Invalid tier"));
        }
    }
    db::upsert_personal_goal(&surreal, &user.id, &goal_type, &target_value).await.map_err(|e| ServerFnError::new(e.to_string()))
}

fn window_to_cutoff(window: &str) -> Option<String> {
    use chrono::{Duration, Utc};
    let days: i64 = match window { "7d" => 7, "30d" => 30, "90d" => 90, _ => return None };
    Some((Utc::now() - Duration::days(days)).to_rfc3339())
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
// Phase 18-08: Mode toggle persistence
// ---------------------------------------------------------------------------

/// Persist the user's solo mode preference. Validates against allowlist before DB write.
/// Mitigates T-18-08-01 (tampering via arbitrary mode string injection).
#[server]
pub async fn set_solo_mode_pref(mode: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    // App-layer validation — DB has no ASSERT per Research Pitfall 4
    const VALID: &[&str] = &["auto", "constellation", "forge", "journal"];
    if !VALID.contains(&mode.as_str()) {
        return Err(ServerFnError::new("Invalid solo mode"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_solo_mode(&db, &user.id, &mode)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Resolve the effective solo mode from a stored preference and region context.
/// Returns region-coupled defaults when stored == "auto" (D-04).
/// An explicit user pick (stored != "auto") always wins over the default (D-05).
fn resolve_mode(stored: &str, region: &str) -> String {
    if stored != "auto" {
        return stored.to_string();
    }
    match region {
        "pandemonium" => "forge".to_string(),
        _ => "constellation".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn SoloDashboardPage() -> impl IntoView {
    // Region read ONCE at page entry — passed as prop to all sub-components.
    // Per SPEC Constraints: InitialTheme context must NOT be read inside primitives.
    let theme = use_context::<InitialTheme>().unwrap_or_default();
    let region = theme.0.clone();

    // Auth redirect + user resource (also used for mode preference)
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

    // Mode preference — signal-driven, persisted via set_solo_mode_pref server fn (18-08)
    let (mode_current, set_mode_current) = signal(
        resolve_mode("auto", &region)
    );
    let region_for_mode = region.clone();
    Effect::new(move |_| {
        if let Some(Ok(Some(user))) = auth_user.get() {
            let resolved = resolve_mode(&user.solo_mode, &region_for_mode);
            set_mode_current.set(resolved);
        }
    });
    let set_mode_action = Action::new(move |new_mode: &String| {
        let m = new_mode.clone();
        async move { set_solo_mode_pref(m).await }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = set_mode_action.value().get() {
            auth_user.refetch();
        }
    });
    let on_mode_select = Callback::new(move |new_mode: String| {
        set_mode_current.set(new_mode.clone());
        set_mode_action.dispatch(new_mode);
    });

    let queue_filter: RwSignal<Option<i32>> = RwSignal::new(None);
    let dashboard_resource = Resource::new(move || queue_filter.get(), |qf| get_solo_dashboard(qf));

    let goal_progress_resource = Resource::new(|| (), |_| async move { compute_goal_progress().await });

    let lp_window: RwSignal<&'static str> = RwSignal::new("30d");
    let lp_history_resource = Resource::new(
        move || lp_window.get(),
        |w| async move { get_lp_history(w.to_string()).await },
    );

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
                            goal_progress_resource.refetch();
                            lp_history_resource.refetch();
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
                    goal_progress_resource.refetch();
                    lp_history_resource.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
            syncing.set(false);
        });
    };

    let region_for_header = region.clone();
    let region_for_suspense = region.clone();
    let region_for_toggle = region.clone();

    view! {
        <div class="canvas-grain bg-base min-h-screen px-8 py-6">
            <div class="max-w-3xl mx-auto flex flex-col gap-8">

                // ── Solo-Constellation Header — region-aware ─────────────────────
                <Card region=region_for_header.clone()
                      variant=if region_for_header == "demacia" { "gilt" } else { "zine" }>
                    <div class="flex items-end justify-between">
                        <SectionHead region=region_for_header.clone()
                                     eyebrow=if region_for_header == "demacia" { "STARS ALIGN" } else { "// SOLO_PROFILE" }
                                     title=if region_for_header == "demacia" { "Constellation".to_string() } else { "FORGE".to_string() }
                        />
                        <button
                            class=move || if syncing.get() {
                                "bg-accent opacity-60 cursor-not-allowed text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            } else {
                                "bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
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
                    // Mode toggle — Constellation / Forge / Journal (18-08)
                    <div class="mt-3">
                        <ModeToggle
                            region=region_for_toggle.clone()
                            current=mode_current
                            options=vec![
                                ("constellation".to_string(), "Constellation".to_string(), "CONSTELLATION".to_string()),
                                ("forge".to_string(), "Forge".to_string(), "FORGE".to_string()),
                                ("journal".to_string(), "Journal".to_string(), "JOURNAL".to_string()),
                            ]
                            on_select=on_mode_select
                        />
                    </div>
                </Card>

                // ── Ranked Badge + LP History + Matches + Goals ─────────────────
                <Suspense fallback={
                    let r = region_for_suspense.clone();
                    move || view! { <PageLoading region=r.clone() variant="solo".to_string() /> }
                }>
                    {move || dashboard_resource.get().map(|result| match result {
                        Err(e) => view! {
                            <ErrorBanner message=format!("Failed to load dashboard: {e}") />
                        }.into_any(),
                        Ok(data) => {
                            let region_inner = region.clone();
                            let region_lp = region.clone();
                            let region_goals = region.clone();
                            let region_mode = region.clone();

                            // Mode dispatch — signal-driven (18-08: DB persistence + resolve_mode)
                            match mode_current.get().as_str() {
                                "forge" => view! {
                                    <SoloForgeView
                                        region=region_mode
                                        ranked=data.ranked
                                    />
                                }.into_any(),
                                "journal" => view! {
                                    <SoloJournalView
                                        region=region_mode
                                        matches=data.matches
                                    />
                                }.into_any(),
                                _ => view! {
                                    // SoloConstellationContent: extracted sub-view to avoid FnOnce
                                    // closures (plan step 8 — sub-view extraction for recursion/closure safety).
                                    <SoloConstellationContent
                                        region=region_inner
                                        ranked=data.ranked
                                        matches=data.matches
                                        lp_history_resource=lp_history_resource
                                        lp_window=lp_window
                                        queue_filter=queue_filter
                                        goal_progress_resource=goal_progress_resource
                                        lp_region=region_lp
                                        goals_region=region_goals
                                    />
                                }.into_any(),
                            }
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

// RankedBadgeSection is no longer the top-level rank display — the rank info is
// now embedded inline inside SoloDashboardPage using RankBadge + LPProgress primitives.
// Kept here for reference but not currently used.
#[allow(dead_code)]
#[component]
fn RankedBadgeSection(ranked: Option<RankedInfo>) -> impl IntoView {
    view! {
        <div class="bg-elevated border border-outline rounded-xl p-6">
            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-accent mb-3">"Ranked Solo / Duo"</div>
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
                        <div class="flex items-center gap-5">
                            <img
                                src=tier_emblem_url(&tier_for_img)
                                alt=tier.clone()
                                class="w-20 h-20 object-contain"
                                style="filter: drop-shadow(0 0 12px color-mix(in oklab, var(--color-accent) 35%, transparent))"
                            />
                            <div class="flex flex-col gap-1">
                                <span class="font-display italic text-primary text-[32px] leading-tight">{display_name}</span>
                                <span class="font-mono text-2xl text-accent tabular-nums">{format!("{} LP", info.lp)}</span>
                                <span class="text-sm text-muted font-mono tabular-nums">
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
                            aria-hidden="true"
                        >
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
                        </svg>
                        <span class="font-display italic text-2xl text-muted">"Unranked"</span>
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
    region: String,
) -> impl IntoView {
    let region_head = region.clone();
    view! {
        <div class="flex flex-col gap-3">
            // Section header
            <div class="flex items-center justify-between">
                <SectionHead region=region_head.clone()
                             eyebrow="BATTLE LOG"
                             title="Recent Matches".to_string() />
                <select
                    class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-sm text-secondary focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
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
                    <PageEmpty region=region.clone() kind="matches".to_string() />
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col gap-2">
                        {matches.into_iter().map(|m| {
                            let border_class = if m.win {
                                "border-l-4 border-info"
                            } else {
                                "border-l-4 border-danger/60"
                            };
                            let row_class = format!(
                                "bg-elevated border border-outline/50 {} rounded-xl p-3 flex items-center gap-3 cursor-pointer hover:bg-surface transition-colors",
                                border_class
                            );
                            let kda = format!("{}/{}/{}", m.kills, m.deaths, m.assists);
                            let cs_str = format!("{} CS", m.cs);
                            let match_href = format!("/match/{}", m.match_id);

                            view! {
                                <a href=match_href class="block cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-xl">
                                    <div class=row_class>
                                        <span class="text-sm font-medium text-primary flex-1">{m.champion}</span>
                                        <span class="text-sm text-secondary font-mono tabular-nums">{kda}</span>
                                        <span class="text-xs text-muted font-mono tabular-nums">{cs_str}</span>
                                    </div>
                                </a>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// LP History Graph
// ---------------------------------------------------------------------------

#[component]
fn LpHistoryGraph(
    lp_history_resource: Resource<Result<Vec<RankedSnapshot>, ServerFnError>>,
    lp_window: RwSignal<&'static str>,
    region: String,
) -> impl IntoView {
    let tooltip: RwSignal<Option<(f64, f64, String, String)>> = RwSignal::new(None);

    let render_pill = move |w: &'static str| {
        let active = move || lp_window.get() == w;
        view! {
            <button
                class=move || if active() {
                    "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                } else {
                    "bg-surface border border-outline/50 text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                }
                on:click=move |_| lp_window.set(w)
            >{w}</button>
        }
    };

    let region_card = region.clone();
    view! {
        <Card region=region_card.clone()>
            <div class="flex items-center justify-between">
                <SectionHead region=region.clone()
                             eyebrow="FORM CURVE"
                             title="LP History".to_string() />
                <div class="flex items-center gap-2">
                    {render_pill("7d")}
                    {render_pill("30d")}
                    {render_pill("90d")}
                    {render_pill("All-time")}
                </div>
            </div>
            <Suspense fallback={
                let r = region.clone();
                move || view! { <PageLoading region=r.clone() variant="solo".to_string() /> }
            }>
                {move || lp_history_resource.get().map(|result| match result {
                    Err(_) => view! {
                        <ErrorBanner message="Could not load LP history. Refresh to try again.".to_string() />
                    }.into_any(),
                    Ok(snaps) if snaps.is_empty() => view! {
                        <div class="flex flex-col items-center justify-center py-8 gap-3 text-center">
                            <div class="text-primary text-sm font-semibold">"No ranked data yet"</div>
                            <div class="text-muted text-sm">"Sync your match history to start tracking LP over time."</div>
                        </div>
                    }.into_any(),
                    Ok(snaps) => view! {
                        <LpGraphSvg snapshots=snaps tooltip=tooltip />
                    }.into_any(),
                })}
            </Suspense>
        </Card>
    }
}

#[component]
fn LpGraphSvg(
    snapshots: Vec<RankedSnapshot>,
    tooltip: RwSignal<Option<(f64, f64, String, String)>>,
) -> impl IntoView {
    const SVG_W: f64 = 800.0;
    const SVG_H: f64 = 160.0;
    const Y_AXIS_W: f64 = 48.0;
    const X_AXIS_H: f64 = 24.0;

    if snapshots.is_empty() {
        return view! { <div /> }.into_any();
    }

    let n = snapshots.len();
    let min_score = snapshots.iter().map(|s| s.rank_score).min().unwrap_or(0) as f64;
    let max_score = snapshots.iter().map(|s| s.rank_score).max().unwrap_or(1) as f64;
    let span = (max_score - min_score).max(1.0);
    let x_step = if n > 1 { (SVG_W - Y_AXIS_W) / (n - 1) as f64 } else { 0.0 };
    let y_scale = (SVG_H - X_AXIS_H) / span;
    let bottom_y = SVG_H - X_AXIS_H;

    let points: Vec<(f64, f64)> = snapshots.iter().enumerate().map(|(i, s)| {
        let x = if n > 1 { Y_AXIS_W + i as f64 * x_step } else { Y_AXIS_W + (SVG_W - Y_AXIS_W) / 2.0 };
        let y = bottom_y - (s.rank_score as f64 - min_score) * y_scale;
        (x, y)
    }).collect();

    let line_points = points.iter().map(|(x, y)| format!("{:.1},{:.1}", x, y)).collect::<Vec<_>>().join(" ");

    let area_d = if n >= 2 {
        let mut d = format!("M {:.1},{:.1}", points[0].0, points[0].1);
        for (x, y) in points.iter().skip(1) { d.push_str(&format!(" L {:.1},{:.1}", x, y)); }
        d.push_str(&format!(" L {:.1},{:.1} L {:.1},{:.1} Z", points.last().unwrap().0, bottom_y, points[0].0, bottom_y));
        d
    } else { String::new() };

    #[allow(unused_variables)]
    let snaps_for_hover = snapshots.clone();
    #[allow(unused_variables)]
    let points_for_hover = points.clone();

    let tier_labels = {
        let boundaries: &[(i32, &str)] = &[
            (0, "Iron"), (400, "Bronze"), (800, "Silver"), (1200, "Gold"),
            (1600, "Platinum"), (2000, "Emerald"), (2400, "Diamond"), (2800, "Master"),
        ];
        let mut out: Vec<(f64, String)> = boundaries.iter()
            .filter(|(s, _)| (*s as f64) >= min_score - 100.0 && (*s as f64) <= max_score + 100.0)
            .map(|(s, name)| { let y = bottom_y - (*s as f64 - min_score) * y_scale; (y, name.to_string()) })
            .collect();
        if out.len() > 5 { out = out.into_iter().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, v)| v).collect(); }
        out
    };

    let date_labels: Vec<(f64, String)> = if n >= 2 {
        let first_x = points[0].0;
        let last_x = points[n - 1].0;
        let mid_idx = n / 2;
        vec![
            (first_x, snapshots[0].snapshotted_at.get(..10).unwrap_or("").to_string()),
            ((first_x + last_x) / 2.0, snapshots[mid_idx].snapshotted_at.get(..10).unwrap_or("").to_string()),
            (last_x, snapshots[n - 1].snapshotted_at.get(..10).unwrap_or("").to_string()),
        ]
    } else {
        vec![(points[0].0, snapshots[0].snapshotted_at.get(..10).unwrap_or("").to_string())]
    };

    view! {
        <div class="relative w-full">
            <svg
                viewBox=format!("0 0 {} {}", SVG_W, SVG_H)
                preserveAspectRatio="none"
                style="width: 100%; height: 160px; display: block;"
                on:mousemove=move |ev| {
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(target) = ev.target() {
                            if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                                if let Some(svg_el) = el.closest("svg").ok().flatten()
                                    .and_then(|e| e.dyn_into::<web_sys::SvgsvgElement>().ok())
                                {
                                    let rect = svg_el.get_bounding_client_rect();
                                    let scale_x = SVG_W / rect.width().max(1.0);
                                    let scale_y = SVG_H / rect.height().max(1.0);
                                    let cx = (ev.client_x() as f64 - rect.left()) * scale_x;
                                    let cy = (ev.client_y() as f64 - rect.top()) * scale_y;
                                    let mut best: Option<(f64, usize)> = None;
                                    for (i, (px, py)) in points_for_hover.iter().enumerate() {
                                        let dx = px - cx;
                                        let dy = py - cy;
                                        let dist = (dx * dx + dy * dy).sqrt();
                                        if dist < 24.0 && best.map(|b| dist < b.0).unwrap_or(true) {
                                            best = Some((dist, i));
                                        }
                                    }
                                    if let Some((_, idx)) = best {
                                        let snap = &snaps_for_hover[idx];
                                        let (px, py) = points_for_hover[idx];
                                        let label = if matches!(snap.tier.to_uppercase().as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER") {
                                            format!("{} — {} LP", snap.tier.to_uppercase(), snap.lp)
                                        } else {
                                            format!("{} {} — {} LP", snap.tier.to_uppercase(), snap.division, snap.lp)
                                        };
                                        let date = snap.snapshotted_at.get(..10).unwrap_or("").to_string();
                                        let tip_x = px / scale_x;
                                        let tip_y = py / scale_y;
                                        tooltip.set(Some((tip_x, tip_y, label, date)));
                                    } else {
                                        tooltip.set(None);
                                    }
                                }
                            }
                        }
                    }
                    let _ = ev;
                }
                on:mouseleave=move |_| tooltip.set(None)
            >
                {tier_labels.into_iter().map(|(y, label)| view! {
                    <text x="4" y=format!("{:.1}", y + 4.0) fill="var(--t-muted)" font-size="11">{label}</text>
                    <line x1=format!("{}", Y_AXIS_W) y1=format!("{:.1}", y)
                          x2=format!("{}", SVG_W) y2=format!("{:.1}", y)
                          stroke="var(--t-divider)" stroke-width="1" opacity="0.3" />
                }).collect_view()}

                {date_labels.into_iter().map(|(x, label)| view! {
                    <text x=format!("{:.1}", x) y=format!("{:.1}", SVG_H - 6.0)
                          text-anchor="middle" fill="var(--t-muted)" font-size="11">{label}</text>
                }).collect_view()}

                {(!area_d.is_empty()).then(|| view! {
                    <path d=area_d style="fill: var(--color-accent)" opacity="0.1" />
                })}

                {(n >= 2).then(|| view! {
                    <polyline points=line_points fill="none"
                              style="stroke: var(--color-accent)"
                              stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
                })}

                {points.iter().map(|(x, y)| view! {
                    <circle cx=format!("{:.1}", x) cy=format!("{:.1}", y) r="3"
                            style="fill: var(--color-accent)" />
                }).collect_view()}
            </svg>

            {move || tooltip.get().map(|(x, y, label, date)| {
                let flip = x > 480.0;
                let style = if flip {
                    format!("position: absolute; right: {}px; top: {}px; pointer-events: none;", (800.0 - x + 8.0).max(0.0) as i32, (y - 50.0).max(0.0) as i32)
                } else {
                    format!("position: absolute; left: {}px; top: {}px; pointer-events: none;", (x + 8.0) as i32, (y - 50.0).max(0.0) as i32)
                };
                view! {
                    <div class="absolute bg-elevated border border-outline rounded-lg px-3 py-2 shadow-lg z-10 min-w-32" style=style>
                        <div class="text-sm text-primary font-semibold">{label}</div>
                        <div class="text-xs text-muted font-mono">{date}</div>
                    </div>
                }
            })}
        </div>
    }.into_any()
}

const RANK_TIERS: &[&str] = &["IRON", "BRONZE", "SILVER", "GOLD", "PLATINUM", "EMERALD", "DIAMOND", "MASTER", "GRANDMASTER", "CHALLENGER"];
const RANK_DIVISIONS: &[&str] = &["IV", "III", "II", "I"];

fn tier_label_display(t: &str) -> &'static str {
    match t.to_uppercase().as_str() {
        "IRON" => "Iron", "BRONZE" => "Bronze", "SILVER" => "Silver",
        "GOLD" => "Gold", "PLATINUM" => "Platinum", "EMERALD" => "Emerald",
        "DIAMOND" => "Diamond", "MASTER" => "Master",
        "GRANDMASTER" => "Grandmaster", "CHALLENGER" => "Challenger",
        _ => "Unranked",
    }
}

// ---------------------------------------------------------------------------
// Solo Constellation Content — extracted sub-view (plan 18-05 step 8)
// Extracted to avoid FnOnce capture in the Suspense reactive closure.
// All owned String data is passed as props; Resources are passed by copy.
// ---------------------------------------------------------------------------

#[component]
#[allow(clippy::too_many_arguments)]
fn SoloConstellationContent(
    region: String,
    ranked: Option<RankedInfo>,
    matches: Vec<PlayerMatchStats>,
    lp_history_resource: Resource<Result<Vec<RankedSnapshot>, ServerFnError>>,
    lp_window: RwSignal<&'static str>,
    queue_filter: RwSignal<Option<i32>>,
    goal_progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>,
    lp_region: String,
    goals_region: String,
) -> impl IntoView {
    let is_demacia = region == "demacia";
    let is_pandemonium = region == "pandemonium";

    let current_tier = ranked.as_ref().map(|r| r.tier.clone()).unwrap_or_else(|| "UNRANKED".to_string());
    let current_division = ranked.as_ref().map(|r| r.division.clone()).unwrap_or_default();
    let current_lp = ranked.as_ref().map(|r| r.lp).unwrap_or(0);

    // Last-10 W/L for Demacia patch
    let last10: Vec<bool> = matches.iter().take(10).map(|m| m.win).collect();
    let last10_empty = last10.is_empty();

    let region_badge = region.clone();
    let current_tier_pan = current_tier.clone();
    let region_sort = region.clone();
    let region_pool = region.clone();

    view! {
        <div class="flex flex-col gap-8">

            // ── Rank crest + LP progress ──────────────────────────────────────
            <Card region=region.clone()
                  variant=if region == "demacia" { "gilt" } else { "zine" }>
                <div class="flex items-center gap-6">
                    <RankBadge tier=current_tier.clone() division=current_division.clone() large=true />
                    <LPProgress region=region.clone() lp={current_lp.max(0) as u32} max=100 />
                </div>

                // ── PANDEMONIUM PATCH (a): Tier crest — TIER label ──────────
                {is_pandemonium.then(|| view! {
                    <div class="flex items-center gap-3 mt-4">
                        <Glitch region="pandemonium".to_string()>
                            {format!("// TIER \u{00b7} {}", current_tier_pan.to_uppercase())}
                        </Glitch>
                    </div>
                }.into_any())}
            </Card>

            // ── PANDEMONIUM PATCH (b): 4 deep stat cards (2×2 grid) ──────────
            // TODO: Wire real KDA/CS/DMG/Vision values from dashboard_resource
            // when per-stat aggregation is added in a future analytics phase.
            // Current values are placeholders per CONTENT-CONTRACT-AUDIT.md.
            {is_pandemonium.then(|| view! {
                <div class="grid grid-cols-2 gap-2">
                    <Card region="pandemonium".to_string() variant="zine">
                        <Stat label="KDA".to_string() value="3.42".to_string() delta=0.18_f32 />
                    </Card>
                    <Card region="pandemonium".to_string() variant="zine">
                        <Stat label="CS/min".to_string() value="7.1".to_string() delta={-0.2_f32} />
                    </Card>
                    <Card region="pandemonium".to_string() variant="zine">
                        <Stat label="DMG Share".to_string() value="27.3".to_string() unit="%".to_string() delta=2.1_f32 />
                    </Card>
                    <Card region="pandemonium".to_string() variant="zine">
                        <Stat label="Vision/min".to_string() value="1.4".to_string() delta=0.05_f32 />
                    </Card>
                </div>
            }.into_any())}

            // ── LP History ────────────────────────────────────────────────────
            <LpHistoryGraph lp_history_resource=lp_history_resource lp_window=lp_window region=lp_region />

            // ── DEMACIA PATCH (b): Last-10 W/L sequence ──────────────────────
            {if is_demacia {
                // Build pip views eagerly; wrap in StoredValue so they can be
                // read from the Fn closure generated by view!.
                let pips_stored = StoredValue::new(last10.iter().copied().map(|won| {
                    let pip_class = if won {
                        "w-4 h-4 rounded-full bg-accent"
                    } else {
                        "w-4 h-4 rounded-full bg-danger"
                    };
                    let pip_title = if won { "Win" } else { "Loss" };
                    view! {
                        <div class=pip_class title=pip_title></div>
                    }
                }).collect_view());
                view! {
                    <Card region="demacia".to_string() variant="gilt">
                        <SectionHead region="demacia".to_string()
                                     eyebrow="RECENT"
                                     title="Last 10".to_string() />
                        {if last10_empty {
                            view! {
                                <PageEmpty region="demacia".to_string() kind="matches".to_string() />
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex gap-1 mt-2">
                                    {move || pips_stored.get_value()}
                                </div>
                            }.into_any()
                        }}
                    </Card>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            // ── Recent Matches (full list) ────────────────────────────────────
            <MatchListSection
                matches=matches
                queue_filter=queue_filter
                region=region_badge
            />

            // ── DEMACIA PATCH (c): Sort/filter controls ───────────────────────
            {is_demacia.then(|| view! {
                <div class="flex items-center gap-3">
                    <span class="font-mono text-[10px] uppercase tracking-[0.16em] text-muted">"Sort"</span>
                    <Btn region=region_sort.clone() variant="ghost" size="sm">"By Champion"</Btn>
                    <Btn region=region_sort.clone() variant="ghost" size="sm">"By Queue"</Btn>
                    <Btn region=region_sort.clone() variant="ghost" size="sm">"By Date"</Btn>
                </div>
            }.into_any())}

            // ── DEMACIA PATCH (a): Pool-gap warnings ──────────────────────────
            // TODO: Wire real pool-gap detection from match history
            // in a future analytics phase. Current gaps are
            // placeholder advisories per CONTENT-CONTRACT-AUDIT.md.
            {is_demacia.then(|| view! {
                <Card region=region_pool.clone() variant="gilt">
                    <SectionHead region=region_pool.clone()
                                 eyebrow="REVIEW"
                                 title="Pool Gaps".to_string() />
                    <ul class="space-y-2 mt-2">
                        <li class="flex items-center gap-2">
                            <Badge tone="warning">"Gap"</Badge>
                            <span class="font-display italic text-secondary text-sm">
                                "No reliable engage support \u{2014} consider learning Leona or Rell"
                            </span>
                        </li>
                        <li class="flex items-center gap-2">
                            <Badge tone="warning">"Gap"</Badge>
                            <span class="font-display italic text-secondary text-sm">
                                "Low DPS jungle option \u{2014} pick up Kindred or Lillia"
                            </span>
                        </li>
                        <li class="flex items-center gap-2">
                            <Badge tone="warning">"Gap"</Badge>
                            <span class="font-display italic text-secondary text-sm">
                                "Missing scaling carry \u{2014} practice Kayle or Vladimir"
                            </span>
                        </li>
                    </ul>
                </Card>
            }.into_any())}

            // ── Goals ─────────────────────────────────────────────────────────
            <GoalCards progress_resource=goal_progress_resource region=goals_region />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Goal Cards
// ---------------------------------------------------------------------------

#[component]
fn GoalCards(
    progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>,
    region: String,
) -> impl IntoView {
    let region_head = region.clone();
    let region_suspense = region.clone();
    view! {
        <div class="flex flex-col gap-3">
            <SectionHead region=region_head.clone()
                         eyebrow="CAPTAIN\u{2019}S OATH"
                         title="Goals".to_string() />
            <Suspense fallback={
                let r = region_suspense.clone();
                move || view! { <PageLoading region=r.clone() variant="solo".to_string() /> }
            }>
                {move || progress_resource.get().map(|result| match result {
                    Err(_) => view! {
                        <ErrorBanner message="Could not load goals. Refresh to try again.".to_string() />
                    }.into_any(),
                    Ok(payload) => view! {
                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                            <RankTargetCard payload=payload.clone() progress_resource=progress_resource />
                            <CsGoalCard payload=payload.clone() progress_resource=progress_resource />
                            <DeathsGoalCard payload=payload progress_resource=progress_resource />
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn RankTargetCard(
    payload: GoalProgressPayload,
    progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>,
) -> impl IntoView {
    use crate::models::match_data::rank_score;
    let editing = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let initial_tier = payload.rank.as_ref().and_then(|p| p.goal.target_value.split(':').next().map(|s| s.to_string())).unwrap_or_else(|| "GOLD".to_string());
    let initial_div = payload.rank.as_ref().and_then(|p| p.goal.target_value.splitn(2, ':').nth(1).map(|s| s.to_string())).unwrap_or_else(|| "IV".to_string());
    let tier_edit = RwSignal::new(initial_tier);
    let div_edit = RwSignal::new(initial_div);

    let on_save = move |_| {
        error_msg.set(None);
        let tier = tier_edit.get_untracked();
        let div = if matches!(tier.to_uppercase().as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER") { String::new() } else { div_edit.get_untracked() };
        let target = format!("{}:{}", tier, div);
        leptos::task::spawn_local(async move {
            match upsert_personal_goal("rank_target".to_string(), target).await {
                Ok(_) => { editing.set(false); progress_resource.refetch(); }
                Err(_) => error_msg.set(Some("Error: Failed to save goal. Try again.".to_string())),
            }
        });
    };
    let on_discard = move |_| { editing.set(false); error_msg.set(None); };
    let on_set = move |_| editing.set(true);
    let on_edit = move |_| editing.set(true);
    let payload_clone = payload.clone();

    view! {
        <div class="bg-elevated border border-outline/50 rounded-xl p-4 flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <svg class="w-5 h-5 text-dimmed" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                </svg>
                <span class="font-imperial uppercase tracking-[0.18em] text-[11px] text-accent">"Rank Target"</span>
            </div>
            {move || if editing.get() {
                let is_master = matches!(tier_edit.get().to_uppercase().as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER");
                view! {
                    <div class="border-t border-divider mt-3 pt-3 flex flex-col gap-2">
                        <label class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Tier"</label>
                        <select class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors w-full"
                                on:change=move |ev| tier_edit.set(event_target_value(&ev))>
                            {RANK_TIERS.iter().map(|t| {
                                let selected = tier_edit.get_untracked().to_uppercase() == *t;
                                view! { <option value=*t selected=selected>{tier_label_display(t)}</option> }
                            }).collect_view()}
                        </select>
                        {(!is_master).then(|| view! {
                            <label class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Division"</label>
                            <select class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors w-full"
                                    on:change=move |ev| div_edit.set(event_target_value(&ev))>
                                {RANK_DIVISIONS.iter().map(|d| {
                                    let selected = div_edit.get_untracked() == *d;
                                    view! { <option value=*d selected=selected>{*d}</option> }
                                }).collect_view()}
                            </select>
                        })}
                        <div class="flex items-center gap-2 mt-2">
                            <button class="bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-accent-hover transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none" on:click=on_save>"Save Goal"</button>
                            <button class="text-muted hover:text-secondary text-xs focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1" on:click=on_discard>"Discard"</button>
                        </div>
                        {move || error_msg.get().map(|m| view! { <StatusMessage message=m /> })}
                    </div>
                }.into_any()
            } else if payload_clone.rank.is_none() {
                view! {
                    <button class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-semibold px-3 py-1.5 rounded-lg mt-2 self-start transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none" on:click=on_set>"Set Goal"</button>
                }.into_any()
            } else {
                let rank_p = payload_clone.rank.clone().unwrap();
                let parts: Vec<&str> = rank_p.goal.target_value.splitn(2, ':').collect();
                let target_tier = parts.first().copied().unwrap_or("").to_string();
                let target_div = parts.get(1).copied().unwrap_or("").to_string();
                let target_score = rank_score(&target_tier, &target_div, 0);
                let current_score: i32 = rank_p.current_value.unwrap_or(0.0) as i32;
                let to_go = (target_score - current_score).max(0);
                let pct: f32 = if target_score > 0 { ((current_score as f32 / target_score as f32) * 100.0).clamp(0.0, 100.0) } else { 0.0 };
                let achieved = rank_p.achieved;
                let current_label = payload_clone.current_rank.as_ref().map(|c| {
                    if matches!(c.tier.to_uppercase().as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER") {
                        format!("{} — {} LP", tier_label_display(&c.tier), c.lp)
                    } else {
                        format!("{} {} — {} LP", tier_label_display(&c.tier), c.division, c.lp)
                    }
                }).unwrap_or_else(|| "—".to_string());
                let target_display = if matches!(target_tier.to_uppercase().as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER") {
                    format!("{}", tier_label_display(&target_tier))
                } else {
                    format!("{} {}", tier_label_display(&target_tier), target_div)
                };
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex items-center justify-between">
                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Target"</div>
                            {achieved.then(|| view! { <span class="bg-success/20 text-success border border-success/30 font-imperial uppercase tracking-[0.18em] text-[10px] rounded-full px-2 py-0.5">"Achieved"</span> })}
                        </div>
                        <div class="text-sm text-secondary">{target_display}</div>
                        <div class="font-display italic text-primary text-2xl mt-1">{current_label}</div>
                        <div class="text-xs text-muted">{format!("{} LP to go", to_go)}</div>
                        <div class="w-full bg-surface border border-outline/30 rounded-full h-2 mt-2 overflow-hidden">
                            <div class=move || if achieved { "h-2 rounded-full bg-success/70 transition-all" } else { "h-2 rounded-full bg-accent/60 transition-all" }
                                 style=format!("width: {:.0}%", pct) />
                        </div>
                        <button class="text-accent hover:text-accent-hover text-xs transition-colors self-end mt-2 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1" on:click=on_edit>"Edit Goal"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn CsGoalCard(
    payload: GoalProgressPayload,
    progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let initial = payload.cs.as_ref().map(|p| p.goal.target_value.clone()).unwrap_or_else(|| "7.0".to_string());
    let target_edit = RwSignal::new(initial);
    let on_save = move |_| {
        error_msg.set(None);
        let raw = target_edit.get_untracked();
        let parsed: Result<f32, _> = raw.parse();
        match parsed {
            Ok(v) if (0.0..=15.0).contains(&v) => {
                let value = format!("{:.1}", v);
                leptos::task::spawn_local(async move {
                    match upsert_personal_goal("cs_per_min".to_string(), value).await {
                        Ok(_) => { editing.set(false); progress_resource.refetch(); }
                        Err(_) => error_msg.set(Some("Error: Failed to save goal. Try again.".to_string())),
                    }
                });
            }
            _ => error_msg.set(Some("Error: Enter a number between 0 and 15.".to_string())),
        }
    };
    let on_discard = move |_| { editing.set(false); error_msg.set(None); };
    let on_set = move |_| editing.set(true);
    let on_edit = move |_| editing.set(true);
    let cs_state = payload.cs.clone();
    view! {
        <div class="bg-elevated border border-outline/50 rounded-xl p-4 flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <svg class="w-5 h-5 text-dimmed" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 18L9 11.25l4.306 4.307a11.95 11.95 0 015.814-5.519l2.74-1.22m0 0l-5.94-2.28m5.94 2.28l-2.28 5.941" />
                </svg>
                <span class="font-imperial uppercase tracking-[0.18em] text-[11px] text-accent">"CS per Minute"</span>
            </div>
            {move || if editing.get() {
                view! {
                    <div class="border-t border-divider mt-3 pt-3 flex flex-col gap-2">
                        <label class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Target CS/min"</label>
                        <input type="number" min="0" max="15" step="0.1"
                               prop:value=move || target_edit.get()
                               on:input=move |ev| target_edit.set(event_target_value(&ev))
                               class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors w-full" />
                        <div class="flex items-center gap-2 mt-2">
                            <button class="bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-accent-hover transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none" on:click=on_save>"Save Goal"</button>
                            <button class="text-muted hover:text-secondary text-xs focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1" on:click=on_discard>"Discard"</button>
                        </div>
                        {move || error_msg.get().map(|m| view! { <StatusMessage message=m /> })}
                    </div>
                }.into_any()
            } else if cs_state.is_none() {
                view! {
                    <button class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-semibold px-3 py-1.5 rounded-lg mt-2 self-start transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none" on:click=on_set>"Set Goal"</button>
                }.into_any()
            } else {
                let p = cs_state.clone().unwrap();
                let target: f32 = p.goal.target_value.parse().unwrap_or(0.0);
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex items-center justify-between">
                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Target"</div>
                            {p.achieved.then(|| view! { <span class="bg-success/20 text-success border border-success/30 font-imperial uppercase tracking-[0.18em] text-[10px] rounded-full px-2 py-0.5">"On track"</span> })}
                        </div>
                        <div class="text-sm text-secondary">{format!("{:.1} CS/min", target)}</div>
                        {match p.current_value {
                            None => view! {
                                <div class="text-sm text-muted mt-2">{format!("Need {} more solo/duo games to track progress", 5 - p.game_count)}</div>
                            }.into_any(),
                            Some(cur) => {
                                let pct = ((cur / target.max(0.01)) * 100.0).clamp(0.0, 100.0);
                                let achieved = p.achieved;
                                view! {
                                    <div class="font-display italic text-primary text-2xl mt-1 tabular-nums">{format!("{:.1}", cur)}</div>
                                    <div class="text-xs text-muted font-mono tabular-nums">{format!("Avg last 20 games: {:.1} / {:.1} target", cur, target)}</div>
                                    <div class="w-full bg-surface border border-outline/30 rounded-full h-2 mt-2 overflow-hidden">
                                        <div class=move || if achieved { "h-2 rounded-full bg-success/70 transition-all" } else { "h-2 rounded-full bg-accent/60 transition-all" }
                                             style=format!("width: {:.0}%", pct) />
                                    </div>
                                }.into_any()
                            }
                        }}
                        <button class="text-accent hover:text-accent-hover text-xs transition-colors self-end mt-2 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1" on:click=on_edit>"Edit Goal"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn DeathsGoalCard(
    payload: GoalProgressPayload,
    progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let initial = payload.deaths.as_ref().map(|p| p.goal.target_value.clone()).unwrap_or_else(|| "4".to_string());
    let target_edit = RwSignal::new(initial);
    let on_save = move |_| {
        error_msg.set(None);
        let raw = target_edit.get_untracked();
        let parsed: Result<i32, _> = raw.parse();
        match parsed {
            Ok(v) if (0..=20).contains(&v) => {
                let value = format!("{}", v);
                leptos::task::spawn_local(async move {
                    match upsert_personal_goal("deaths_per_game".to_string(), value).await {
                        Ok(_) => { editing.set(false); progress_resource.refetch(); }
                        Err(_) => error_msg.set(Some("Error: Failed to save goal. Try again.".to_string())),
                    }
                });
            }
            _ => error_msg.set(Some("Error: Enter a whole number between 0 and 20.".to_string())),
        }
    };
    let on_discard = move |_| { editing.set(false); error_msg.set(None); };
    let on_set = move |_| editing.set(true);
    let on_edit = move |_| editing.set(true);
    let deaths_state = payload.deaths.clone();
    view! {
        <div class="bg-elevated border border-outline/50 rounded-xl p-4 flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <svg class="w-5 h-5 text-dimmed" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                </svg>
                <span class="font-imperial uppercase tracking-[0.18em] text-[11px] text-accent">"Deaths per Game"</span>
            </div>
            {move || if editing.get() {
                view! {
                    <div class="border-t border-divider mt-3 pt-3 flex flex-col gap-2">
                        <label class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Max deaths per game"</label>
                        <input type="number" min="0" max="20" step="1"
                               prop:value=move || target_edit.get()
                               on:input=move |ev| target_edit.set(event_target_value(&ev))
                               class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors w-full" />
                        <div class="flex items-center gap-2 mt-2">
                            <button class="bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-accent-hover transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none" on:click=on_save>"Save Goal"</button>
                            <button class="text-muted hover:text-secondary text-xs focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1" on:click=on_discard>"Discard"</button>
                        </div>
                        {move || error_msg.get().map(|m| view! { <StatusMessage message=m /> })}
                    </div>
                }.into_any()
            } else if deaths_state.is_none() {
                view! {
                    <button class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-semibold px-3 py-1.5 rounded-lg mt-2 self-start transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none" on:click=on_set>"Set Goal"</button>
                }.into_any()
            } else {
                let p = deaths_state.clone().unwrap();
                let target: f32 = p.goal.target_value.parse().unwrap_or(0.0);
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex items-center justify-between">
                            <div class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">"Target"</div>
                            {p.achieved.then(|| view! { <span class="bg-success/20 text-success border border-success/30 font-imperial uppercase tracking-[0.18em] text-[10px] rounded-full px-2 py-0.5">"On track"</span> })}
                        </div>
                        <div class="text-sm text-secondary">{format!("{:.0} deaths or fewer", target)}</div>
                        {match p.current_value {
                            None => view! {
                                <div class="text-sm text-muted mt-2">{format!("Need {} more solo/duo games to track progress", 5 - p.game_count)}</div>
                            }.into_any(),
                            Some(cur) => {
                                let pct = if cur <= target { 100.0f32 } else { ((target / cur.max(0.01)) * 100.0).clamp(0.0, 100.0) };
                                let achieved = p.achieved;
                                view! {
                                    <div class="font-display italic text-primary text-2xl mt-1 tabular-nums">{format!("{:.1}", cur)}</div>
                                    <div class="text-xs text-muted font-mono tabular-nums">{format!("Avg last 20 games: {:.1} / {:.1} target", cur, target)}</div>
                                    <div class="w-full bg-surface border border-outline/30 rounded-full h-2 mt-2 overflow-hidden">
                                        <div class=move || if achieved { "h-2 rounded-full bg-success/70 transition-all" } else { "h-2 rounded-full bg-accent/60 transition-all" }
                                             style=format!("width: {:.0}%", pct) />
                                    </div>
                                }.into_any()
                            }
                        }}
                        <button class="text-accent hover:text-accent-hover text-xs transition-colors self-end mt-2 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded-md px-2 py-1" on:click=on_edit>"Edit Goal"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}


// ---------------------------------------------------------------------------
// SoloJournalView — 18-07
// Personal journal / match diary sub-view for /solo route mode="journal".
// Demacia: parchment diary (gilt Cards, Cormorant serif entries)
// Pandemonium: photocopied fanzine (RiotTape, zine Cards, rotate transforms)
//
// ChildrenFn constraints (Card, Glitch both use ChildrenFn):
//   - All String props in ChildrenFn bodies must use .clone() to remain Fn.
//   - All String values used in multiple closures must each get their own
//     StoredValue<String> so closures capture a Copy type and remain Fn.
// ---------------------------------------------------------------------------

/// Entry data for a journal row.
#[derive(Clone)]
struct JournalEntry {
    date_label: String,
    body: String,
    won: bool,
}

#[component]
fn SoloJournalView(
    region: String,
    /// Recent matches used as journal entry source.
    matches: Vec<PlayerMatchStats>,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let matches_empty = matches.is_empty();

    let entries: Vec<JournalEntry> = matches.iter().take(5).map(|m| {
        let outcome = if m.win { "Victory" } else { "Defeat" };
        let body = format!("{} — {} · KDA {}/{}/{}", outcome, m.champion, m.kills, m.deaths, m.assists);
        JournalEntry { date_label: "Recent".to_string(), body, won: m.win }
    }).collect();

    // StoredValue<Vec<JournalEntry>> is Copy+Send+Sync; get_value() returns a clone each call.
    let entries_sv = StoredValue::new(entries);

    if is_pandemonium {
        let r_empty = region.clone();
        // r_footer: used for Card region prop (clone) and Glitch region (another clone inside).
        let r_footer = region.clone();
        view! {
            <div class="flex flex-col gap-3">
                <RiotTape label="JOURNAL_RAW" />
                {if matches_empty {
                    view! {
                        <PageEmpty region=r_empty kind="matches".to_string() />
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col gap-3 mt-2">
                            // Each field uses its own StoredValue to avoid double-move of e.
                            {move || entries_sv.get_value().into_iter().enumerate().map(|(i, e)| {
                                let rotate_class = if i % 2 == 0 { "-rotate-1" } else { "rotate-1" };
                                let win_label = if e.won { "WIN" } else { "LOSS" };
                                let tone = if e.won { "accent" } else { "neutral" };
                                let date_sv = StoredValue::new(e.date_label);
                                let body_sv = StoredValue::new(e.body);
                                view! {
                                    <div class=format!("transform {}", rotate_class)>
                                        <Card region="pandemonium".to_string() variant="zine">
                                            <Glitch region="pandemonium".to_string()>
                                                {move || format!("// ENTRY_{}", date_sv.get_value())}
                                            </Glitch>
                                            <div class="mt-2 font-mono text-[12px] text-primary leading-relaxed">
                                                {move || body_sv.get_value()}
                                            </div>
                                            <div class="mt-2">
                                                <Badge tone=tone>{win_label}</Badge>
                                            </div>
                                        </Card>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
                // TODO: wire personal_learnings resource when /personal-learnings phase connects
                <Card region=r_footer.clone() variant="zine">
                    <Glitch region=r_footer.clone()>"// DEEPER_NOTES"</Glitch>
                    <pre class="font-mono text-[11px] text-secondary mt-2 whitespace-pre-wrap">
                        "// Extended notes pending.\n// Navigate to /personal-learnings."
                    </pre>
                </Card>
            </div>
        }.into_any()
    } else {
        let r_empty = region.clone();
        // r_hdr: used for Card region (.clone()) and SectionHead (.clone()), retain for further use.
        let r_hdr = region.clone();
        let r_footer = region.clone();
        view! {
            <div class="flex flex-col gap-4">
                <Card region=r_hdr.clone() variant="gilt">
                    <SectionHead
                        region=r_hdr.clone()
                        eyebrow="CHRONICLES"
                        title="Journal".to_string()
                    />
                    <div class="mt-2 flex justify-center">
                        <HeraldicDivider width=480 />
                    </div>
                </Card>
                {if matches_empty {
                    view! {
                        <PageEmpty region=r_empty kind="matches".to_string() />
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col gap-3">
                            {move || entries_sv.get_value().into_iter().map(|e| {
                                let mood_class = if e.won {
                                    "w-1 rounded-full bg-accent/60 self-stretch shrink-0"
                                } else {
                                    "w-1 rounded-full bg-secondary/30 self-stretch shrink-0"
                                };
                                let date_sv = StoredValue::new(e.date_label);
                                let body_sv = StoredValue::new(e.body);
                                view! {
                                    <Card region="demacia".to_string() variant="gilt">
                                        <div class="flex gap-3">
                                            <div class=mood_class></div>
                                            <div class="flex-1">
                                                <div class="font-imperial text-[9px] uppercase tracking-[0.2em] text-muted mb-1">
                                                    {move || date_sv.get_value()}
                                                </div>
                                                <p class="font-display italic text-secondary text-[13px] leading-relaxed">
                                                    {move || body_sv.get_value()}
                                                </p>
                                            </div>
                                        </div>
                                    </Card>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
                // TODO: wire personal_learnings resource when /personal-learnings phase connects
                <Card region=r_footer.clone() variant="gilt">
                    <Eyebrow>"EXTENDED CHRONICLES"</Eyebrow>
                    <p class="font-display italic text-secondary mt-2 text-[13px]">
                        "Navigate to /personal-learnings to document deeper match reflections."
                    </p>
                </Card>
            </div>
        }.into_any()
    }
}

// ---------------------------------------------------------------------------
// SoloForgeView — 18-07
// Champion mastery / queue-prep sub-view for /solo route mode="forge".
// Demacia: smith's workbench (gilt Cards, Crown, Cormorant typography)
// Pandemonium: locker/workbench (RiotTape, zine Cards, mono checklist)
// ---------------------------------------------------------------------------

#[component]
fn SoloForgeView(
    region: String,
    ranked: Option<RankedInfo>,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";

    let current_tier = ranked.as_ref().map(|r| r.tier.clone()).unwrap_or_else(|| "UNRANKED".to_string());
    let current_division = ranked.as_ref().map(|r| r.division.clone()).unwrap_or_default();
    let current_lp = ranked.as_ref().map(|r| r.lp).unwrap_or(0);
    let tier_upper = current_tier.to_uppercase();

    // TODO: wire real prep/pool data from champion pool + match history phases
    // StoredValue<Vec<[String; N]>> — each String is Send+Sync+Clone.
    let prep_sv = StoredValue::new(vec![
        ["[ ]".to_string(), "Aatrox — review lvl 3 jungle clear".to_string()],
        ["[ ]".to_string(), "Study: how to play into poke comps".to_string()],
        ["[x]".to_string(), "Warmup: 20 CS practice tool".to_string()],
        ["[ ]".to_string(), "Mental reset — previous tilt noted".to_string()],
    ]);

    // TODO: wire from champion pool resource once pool-fill aggregation is added
    let pool_sv = StoredValue::new(vec![
        ["Aatrox".to_string(),   "12 games".to_string(), "58% WR".to_string()],
        ["Jax".to_string(),      "8 games".to_string(),  "62% WR".to_string()],
        ["Malphite".to_string(), "5 games".to_string(),  "40% WR".to_string()],
    ]);

    let targets_sv = StoredValue::new(vec![
        ["Aatrox".to_string(),   "Master carry patterns".to_string()],
        ["Jax".to_string(),      "Refine split-push decision-making".to_string()],
        ["Malphite".to_string(), "Consistent engage timing".to_string()],
    ]);

    let champ_pool_sv = StoredValue::new(vec![
        "Aatrox".to_string(), "Jax".to_string(), "Malphite".to_string(),
    ]);

    if is_pandemonium {
        let tier_pan = StoredValue::new(current_tier.clone());
        let div_pan  = StoredValue::new(current_division.clone());
        let tier_up_pan = StoredValue::new(tier_upper.clone());
        let r1 = region.clone();
        let r2 = region.clone();
        let r3 = region.clone();
        let r4 = region.clone();
        let r5 = region.clone();
        let r6 = region.clone();
        let r7 = region.clone();
        view! {
            <div class="flex flex-col gap-3">
                <RiotTape label="FORGE · QUEUE_PREP" />
                // Rank display
                <Card region=r1 variant="zine">
                    <Glitch region=r2.clone()>"// TIER"</Glitch>
                    <div class="flex items-center gap-4 mt-2">
                        <RankBadge tier=tier_pan.get_value() division=div_pan.get_value() large=false />
                        <LPProgress region=r3.clone() lp={current_lp.max(0) as u32} max=100 />
                    </div>
                    <div class="mt-2">
                        <Glitch region=r4.clone()>
                            {move || format!("// TIER · {}", tier_up_pan.get_value())}
                        </Glitch>
                    </div>
                </Card>
                // Queue prep checklist
                <Card region=r5 variant="zine">
                    <Glitch region=r6.clone()>"// PREP_LIST"</Glitch>
                    <div class="mt-2 flex flex-col gap-1">
                        {move || prep_sv.get_value().into_iter().map(|row| {
                            let a_sv = StoredValue::new(row[0].clone());
                            let b_sv = StoredValue::new(row[1].clone());
                            view! {
                                <div class="font-mono text-[12px] flex gap-2 items-baseline border-b border-outline/20 py-1">
                                    <span class="text-accent shrink-0">{move || a_sv.get_value()}</span>
                                    <span class="text-primary">{move || b_sv.get_value()}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </Card>
                // Pool status
                <Card region=r7 variant="zine">
                    <Glitch region=region.clone()>"// POOL_STATUS"</Glitch>
                    <div class="mt-2 flex flex-col gap-1">
                        {move || pool_sv.get_value().into_iter().map(|row| {
                            let a_sv = StoredValue::new(row[0].clone());
                            let b_sv = StoredValue::new(row[1].clone());
                            let c_sv = StoredValue::new(row[2].clone());
                            view! {
                                <div class="font-mono text-[11px] flex gap-2 items-baseline border-b border-outline/20 py-1">
                                    <span class="text-primary flex-1">{move || a_sv.get_value()}</span>
                                    <span class="text-secondary">{move || b_sv.get_value()}</span>
                                    <span class="text-accent">{move || c_sv.get_value()}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </Card>
                // CTA
                <Btn region="pandemonium".to_string() variant="primary">
                    "QUEUE"
                </Btn>
            </div>
        }.into_any()
    } else {
        let tier_dem = StoredValue::new(current_tier.clone());
        let div_dem  = StoredValue::new(current_division.clone());
        let r1 = region.clone();
        let r2 = region.clone();
        let r3 = region.clone();
        let r4 = region.clone();
        let r5 = region.clone();
        let r6 = region.clone();
        let r7 = region.clone();
        let r8 = region.clone();
        view! {
            <div class="flex flex-col gap-4">
                // Forge header
                <Card region=r1.clone() variant="gilt">
                    <div class="text-center pb-2">
                        <div class="flex justify-center mb-2">
                            <Crown size=36 />
                        </div>
                        <h1 class="font-display text-[24px] tracking-[0.08em] text-accent uppercase">
                            "FORGE"
                        </h1>
                        <SectionHead
                            region=r1.clone()
                            eyebrow="CHAMPION MASTERY"
                            title="Workbench".to_string()
                        />
                        <div class="mt-2 flex justify-center">
                            <HeraldicDivider width=400 />
                        </div>
                    </div>
                </Card>
                // Rank display
                <Card region=r2 variant="gilt">
                    <SectionHead
                        region=r3.clone()
                        eyebrow="CURRENT STANDING"
                        title="Rank".to_string()
                    />
                    <div class="flex items-center gap-6 mt-3">
                        <RankBadge tier=tier_dem.get_value() division=div_dem.get_value() large=true />
                        <LPProgress region=r4.clone() lp={current_lp.max(0) as u32} max=100 />
                    </div>
                </Card>
                // Improvement targets
                <Card region=r5 variant="gilt">
                    <SectionHead
                        region=r6.clone()
                        eyebrow="FORGE"
                        title="Improvement Targets".to_string()
                    />
                    <div class="flex justify-center mt-2">
                        <HeraldicDivider width=480 />
                    </div>
                    <div class="mt-3 flex flex-col gap-2">
                        {move || targets_sv.get_value().into_iter().map(|row| {
                            let a_sv = StoredValue::new(row[0].clone());
                            let b_sv = StoredValue::new(row[1].clone());
                            view! {
                                <div class="flex items-baseline gap-3 border-b border-outline/20 py-2 font-display text-[13px]">
                                    <span class="text-accent shrink-0">"•"</span>
                                    <span class="text-primary font-semibold">{move || a_sv.get_value()}</span>
                                    <span class="text-secondary italic flex-1">{move || b_sv.get_value()}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </Card>
                // Tool rack: champion pool
                <Card region=r7 variant="gilt">
                    <SectionHead
                        region=r8.clone()
                        eyebrow="TOOL RACK"
                        title="Champion Pool".to_string()
                    />
                    <div class="flex gap-3 mt-3 flex-wrap">
                        {move || champ_pool_sv.get_value().into_iter().map(|champ| {
                            let champ_badge_sv = StoredValue::new(champ.clone());
                            view! {
                                <div class="flex flex-col items-center gap-1">
                                    <ChampTile name=champ size=56 />
                                    <Badge tone="neutral">{move || champ_badge_sv.get_value()}</Badge>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </Card>
                // Queue CTA
                <Btn region=region variant="primary">
                    "Queue Aatrox"
                </Btn>
            </div>
        }.into_any()
    }
}
