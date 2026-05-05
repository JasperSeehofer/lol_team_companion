use crate::components::ui::{EmptyState, ErrorBanner, SkeletonCard, StatusMessage, ToastContext, ToastKind};
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

    let goal_progress_resource = Resource::new(|| (), |_| async move { compute_goal_progress().await });

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

            // ── Ranked Badge + LP History + Matches + Goals ─────────────────
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
                            <LpHistoryGraph />
                            <MatchListSection
                                matches=data.matches
                                queue_filter=queue_filter
                            />
                            <GoalCards progress_resource=goal_progress_resource />
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
                                "bg-surface {} rounded-lg p-3 flex items-center gap-3 cursor-pointer hover:bg-elevated/50 transition-colors",
                                border_class
                            );
                            let kda = format!("{}/{}/{}", m.kills, m.deaths, m.assists);
                            let cs_str = format!("{} CS", m.cs);
                            let match_href = format!("/match/{}", m.match_id);

                            view! {
                                <a href=match_href class="block cursor-pointer hover:bg-elevated/50 transition-colors">
                                    <div class=row_class>
                                        <span class="text-sm font-medium text-primary flex-1">{m.champion}</span>
                                        <span class="text-sm text-secondary">{kda}</span>
                                        <span class="text-xs text-muted">{cs_str}</span>
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
fn LpHistoryGraph() -> impl IntoView {
    let lp_window: RwSignal<&'static str> = RwSignal::new("30d");
    let lp_history_resource = Resource::new(
        move || lp_window.get(),
        |w| async move { get_lp_history(w.to_string()).await },
    );
    let tooltip: RwSignal<Option<(f64, f64, String, String)>> = RwSignal::new(None);

    let render_pill = move |w: &'static str| {
        let active = move || lp_window.get() == w;
        view! {
            <button
                class=move || if active() {
                    "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                } else {
                    "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                }
                on:click=move |_| lp_window.set(w)
            >{w}</button>
        }
    };

    view! {
        <div class="bg-elevated border border-divider rounded-xl p-4 flex flex-col gap-3">
            <div class="flex items-center justify-between">
                <h2 class="text-xl font-semibold text-primary">"LP History"</h2>
                <div class="flex items-center gap-2">
                    {render_pill("7d")}
                    {render_pill("30d")}
                    {render_pill("90d")}
                    {render_pill("All-time")}
                </div>
            </div>
            <Suspense fallback=|| view! { <SkeletonCard height="h-48" /> }>
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
        </div>
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
                    <path d=area_d fill="var(--t-accent)" opacity="0.1" />
                })}

                {(n >= 2).then(|| view! {
                    <polyline points=line_points fill="none" stroke="var(--t-accent)"
                              stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
                })}

                {points.iter().map(|(x, y)| view! {
                    <circle cx=format!("{:.1}", x) cy=format!("{:.1}", y) r="3"
                            fill="var(--t-accent)" />
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
                    <div class="absolute bg-elevated border border-divider rounded-lg px-3 py-2 shadow-lg z-10 min-w-32" style=style>
                        <div class="text-sm text-primary font-semibold">{label}</div>
                        <div class="text-xs text-muted">{date}</div>
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
// Goal Cards
// ---------------------------------------------------------------------------

#[component]
fn GoalCards(progress_resource: Resource<Result<GoalProgressPayload, ServerFnError>>) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            <h2 class="text-xl font-semibold text-primary">"Goals"</h2>
            <Suspense fallback=|| view! { <SkeletonCard height="h-28" /> }>
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
        <div class="bg-elevated border border-divider/50 rounded-xl p-4 flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <svg class="w-5 h-5 text-dimmed" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                </svg>
                <span class="text-sm font-semibold text-primary">"Rank Target"</span>
            </div>
            {move || if editing.get() {
                let is_master = matches!(tier_edit.get().to_uppercase().as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER");
                view! {
                    <div class="border-t border-divider mt-3 pt-3 flex flex-col gap-2">
                        <label class="text-xs text-muted uppercase tracking-wider">"Tier"</label>
                        <select class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent/50 transition-colors w-full"
                                on:change=move |ev| tier_edit.set(event_target_value(&ev))>
                            {RANK_TIERS.iter().map(|t| {
                                let selected = tier_edit.get_untracked().to_uppercase() == *t;
                                view! { <option value=*t selected=selected>{tier_label_display(t)}</option> }
                            }).collect_view()}
                        </select>
                        {(!is_master).then(|| view! {
                            <label class="text-xs text-muted uppercase tracking-wider">"Division"</label>
                            <select class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent/50 transition-colors w-full"
                                    on:change=move |ev| div_edit.set(event_target_value(&ev))>
                                {RANK_DIVISIONS.iter().map(|d| {
                                    let selected = div_edit.get_untracked() == *d;
                                    view! { <option value=*d selected=selected>{*d}</option> }
                                }).collect_view()}
                            </select>
                        })}
                        <div class="flex items-center gap-2 mt-2">
                            <button class="bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-accent-hover transition-colors" on:click=on_save>"Save Goal"</button>
                            <button class="text-muted hover:text-secondary text-xs" on:click=on_discard>"Discard"</button>
                        </div>
                        {move || error_msg.get().map(|m| view! { <StatusMessage message=m /> })}
                    </div>
                }.into_any()
            } else if payload_clone.rank.is_none() {
                view! {
                    <button class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-semibold px-3 py-1.5 rounded-lg mt-2 self-start transition-colors" on:click=on_set>"Set Goal"</button>
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
                            <div class="text-xs text-muted uppercase tracking-wider">"Target"</div>
                            {achieved.then(|| view! { <span class="bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs font-semibold rounded-full px-2 py-0.5">"Achieved"</span> })}
                        </div>
                        <div class="text-sm text-secondary">{target_display}</div>
                        <div class="text-xl font-semibold text-primary mt-1">{current_label}</div>
                        <div class="text-xs text-muted">{format!("{} LP to go", to_go)}</div>
                        <div class="w-full bg-elevated rounded-full h-2 mt-2">
                            <div class=move || if achieved { "h-2 rounded-full bg-emerald-500/60 transition-all" } else { "h-2 rounded-full bg-accent/50 transition-all" }
                                 style=format!("width: {:.0}%", pct) />
                        </div>
                        <button class="text-accent hover:text-accent-hover text-xs transition-colors self-end mt-2" on:click=on_edit>"Edit Goal"</button>
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
        <div class="bg-elevated border border-divider/50 rounded-xl p-4 flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <svg class="w-5 h-5 text-dimmed" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 18L9 11.25l4.306 4.307a11.95 11.95 0 015.814-5.519l2.74-1.22m0 0l-5.94-2.28m5.94 2.28l-2.28 5.941" />
                </svg>
                <span class="text-sm font-semibold text-primary">"CS per Minute"</span>
            </div>
            {move || if editing.get() {
                view! {
                    <div class="border-t border-divider mt-3 pt-3 flex flex-col gap-2">
                        <label class="text-xs text-muted uppercase tracking-wider">"Target CS/min"</label>
                        <input type="number" min="0" max="15" step="0.1"
                               prop:value=move || target_edit.get()
                               on:input=move |ev| target_edit.set(event_target_value(&ev))
                               class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent/50 transition-colors w-full" />
                        <div class="flex items-center gap-2 mt-2">
                            <button class="bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-accent-hover transition-colors" on:click=on_save>"Save Goal"</button>
                            <button class="text-muted hover:text-secondary text-xs" on:click=on_discard>"Discard"</button>
                        </div>
                        {move || error_msg.get().map(|m| view! { <StatusMessage message=m /> })}
                    </div>
                }.into_any()
            } else if cs_state.is_none() {
                view! {
                    <button class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-semibold px-3 py-1.5 rounded-lg mt-2 self-start transition-colors" on:click=on_set>"Set Goal"</button>
                }.into_any()
            } else {
                let p = cs_state.clone().unwrap();
                let target: f32 = p.goal.target_value.parse().unwrap_or(0.0);
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex items-center justify-between">
                            <div class="text-xs text-muted uppercase tracking-wider">"Target"</div>
                            {p.achieved.then(|| view! { <span class="bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs font-semibold rounded-full px-2 py-0.5">"On track"</span> })}
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
                                    <div class="text-xl font-semibold text-primary mt-1">{format!("{:.1}", cur)}</div>
                                    <div class="text-xs text-muted">{format!("Avg last 20 games: {:.1} / {:.1} target", cur, target)}</div>
                                    <div class="w-full bg-elevated rounded-full h-2 mt-2">
                                        <div class=move || if achieved { "h-2 rounded-full bg-emerald-500/60 transition-all" } else { "h-2 rounded-full bg-accent/50 transition-all" }
                                             style=format!("width: {:.0}%", pct) />
                                    </div>
                                }.into_any()
                            }
                        }}
                        <button class="text-accent hover:text-accent-hover text-xs transition-colors self-end mt-2" on:click=on_edit>"Edit Goal"</button>
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
        <div class="bg-elevated border border-divider/50 rounded-xl p-4 flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <svg class="w-5 h-5 text-dimmed" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                </svg>
                <span class="text-sm font-semibold text-primary">"Deaths per Game"</span>
            </div>
            {move || if editing.get() {
                view! {
                    <div class="border-t border-divider mt-3 pt-3 flex flex-col gap-2">
                        <label class="text-xs text-muted uppercase tracking-wider">"Max deaths per game"</label>
                        <input type="number" min="0" max="20" step="1"
                               prop:value=move || target_edit.get()
                               on:input=move |ev| target_edit.set(event_target_value(&ev))
                               class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent/50 transition-colors w-full" />
                        <div class="flex items-center gap-2 mt-2">
                            <button class="bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-lg font-semibold hover:bg-accent-hover transition-colors" on:click=on_save>"Save Goal"</button>
                            <button class="text-muted hover:text-secondary text-xs" on:click=on_discard>"Discard"</button>
                        </div>
                        {move || error_msg.get().map(|m| view! { <StatusMessage message=m /> })}
                    </div>
                }.into_any()
            } else if deaths_state.is_none() {
                view! {
                    <button class="bg-accent hover:bg-accent-hover text-accent-contrast text-xs font-semibold px-3 py-1.5 rounded-lg mt-2 self-start transition-colors" on:click=on_set>"Set Goal"</button>
                }.into_any()
            } else {
                let p = deaths_state.clone().unwrap();
                let target: f32 = p.goal.target_value.parse().unwrap_or(0.0);
                view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex items-center justify-between">
                            <div class="text-xs text-muted uppercase tracking-wider">"Target"</div>
                            {p.achieved.then(|| view! { <span class="bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs font-semibold rounded-full px-2 py-0.5">"On track"</span> })}
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
                                    <div class="text-xl font-semibold text-primary mt-1">{format!("{:.1}", cur)}</div>
                                    <div class="text-xs text-muted">{format!("Avg last 20 games: {:.1} / {:.1} target", cur, target)}</div>
                                    <div class="w-full bg-elevated rounded-full h-2 mt-2">
                                        <div class=move || if achieved { "h-2 rounded-full bg-emerald-500/60 transition-all" } else { "h-2 rounded-full bg-accent/50 transition-all" }
                                             style=format!("width: {:.0}%", pct) />
                                    </div>
                                }.into_any()
                            }
                        }}
                        <button class="text-accent hover:text-accent-hover text-xs transition-colors self-end mt-2" on:click=on_edit>"Edit Goal"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
