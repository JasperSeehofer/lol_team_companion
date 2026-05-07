use crate::components::ornaments::HeraldicDivider;
use crate::components::stat_card::StatCard;
use crate::components::ui::{EmptyState, ErrorBanner, NoTeamState, SkeletonCard};
use crate::models::match_data::ChampionTrend;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TeamMatchRow {
    pub match_db_id: String,
    pub user_id: String,
    pub username: String,
    pub riot_match_id: String,
    pub queue_id: i32,
    pub game_duration: i32,
    pub game_end: Option<String>,
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub cs: i32,
    pub vision_score: i32,
    pub damage: i32,
    pub win: bool,
}

#[server]
pub async fn check_api_key() -> Result<bool, ServerFnError> {
    use crate::server::riot;
    Ok(riot::has_api_key())
}

#[server]
pub async fn get_team_stats() -> Result<Vec<TeamMatchRow>, ServerFnError> {
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

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let rows = db::get_team_match_stats(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Convert db type to page type
    Ok(rows
        .into_iter()
        .map(|r| TeamMatchRow {
            match_db_id: r.match_db_id,
            user_id: r.user_id,
            username: r.username,
            riot_match_id: r.riot_match_id,
            queue_id: r.queue_id,
            game_duration: r.game_duration,
            game_end: r.game_end,
            champion: r.champion,
            kills: r.kills,
            deaths: r.deaths,
            assists: r.assists,
            cs: r.cs,
            vision_score: r.vision_score,
            damage: r.damage,
            win: r.win,
        })
        .collect())
}

#[server]
pub async fn sync_team_stats(queue_id: Option<i32>) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if !riot::has_api_key() {
        return Err(ServerFnError::new(
            "RIOT_API_KEY not configured. Set it in .env to enable match syncing.",
        ));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => {
            return Err(ServerFnError::new(
                "You need to join or create a team first",
            ))
        }
    };

    let members = db::get_roster_puuids(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let queue_label = match queue_id {
        Some(420) => "Solo/Duo",
        Some(440) => "Flex",
        _ => "All Queues",
    };

    let mut total_synced = 0usize;
    let mut synced_players = 0usize;
    let mut errors = Vec::new();

    for (uid, username, puuid_opt) in &members {
        let puuid = match puuid_opt {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };

        let platform = riot::platform_route_from_str("EUW");
        match riot::fetch_match_history(puuid, queue_id, platform).await {
            Ok(matches) => {
                let count = matches.len();
                if let Err(e) = db::store_matches(&surreal, uid, matches).await {
                    errors.push(format!("{username}: store error: {e}"));
                } else {
                    total_synced += count;
                    synced_players += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{username}: API error: {e}"));
            }
        }
    }

    let linked_count = members
        .iter()
        .filter(|(_, _, p)| p.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
        .count();
    let mut msg = format!("Synced {total_synced} {queue_label} matches for {synced_players}/{linked_count} linked players.");
    if !errors.is_empty() {
        msg.push_str(&format!(" Errors: {}", errors.join("; ")));
    }
    Ok(msg)
}

#[server]
pub async fn get_champion_trends(window: String) -> Result<Vec<ChampionTrend>, ServerFnError> {
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

    let cutoff = trends_window_to_cutoff(&window);
    db::get_champion_trends(&surreal, &user.id, cutoff)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn trends_window_to_cutoff(window: &str) -> Option<String> {
    use chrono::{Duration, Utc};
    let days: i64 = match window {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => return None,
    };
    Some((Utc::now() - Duration::days(days)).to_rfc3339())
}

/// A grouped view of a single match with all participating roster members
#[derive(Clone)]
struct MatchGroup {
    riot_match_id: String,
    queue_id: i32,
    game_end: Option<String>,
    game_duration: i32,
    win: bool,
    players: Vec<TeamMatchRow>,
}

fn group_matches(rows: &[TeamMatchRow]) -> Vec<MatchGroup> {
    let mut by_match: HashMap<String, Vec<TeamMatchRow>> = HashMap::new();
    for r in rows {
        by_match
            .entry(r.riot_match_id.clone())
            .or_default()
            .push(r.clone());
    }

    let mut groups: Vec<MatchGroup> = by_match
        .into_iter()
        .map(|(riot_id, players)| {
            let first = &players[0];
            MatchGroup {
                riot_match_id: riot_id,
                queue_id: first.queue_id,
                game_end: first.game_end.clone(),
                game_duration: first.game_duration,
                win: first.win,
                players,
            }
        })
        .collect();

    // Sort by game_end descending
    groups.sort_by(|a, b| b.game_end.cmp(&a.game_end));
    groups
}

fn format_duration(secs: i32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m}:{s:02}")
}

fn format_date(game_end: &Option<String>) -> String {
    match game_end {
        Some(d) => {
            // Trim to just date portion
            if d.len() >= 10 {
                d[..10].to_string()
            } else {
                d.clone()
            }
        }
        None => "Unknown".to_string(),
    }
}

fn queue_label(queue_id: i32) -> &'static str {
    match queue_id {
        420 => "Solo/Duo",
        440 => "Flex",
        450 => "ARAM",
        400 => "Normal Draft",
        430 => "Normal Blind",
        _ => "Other",
    }
}

/// Champion icon URL from Data Dragon CDN
fn champion_icon_url(champion_name: &str) -> String {
    format!("https://ddragon.leagueoflegends.com/cdn/15.6.1/img/champion/{champion_name}.png")
}

fn format_damage(damage: i32) -> String {
    if damage >= 1000 {
        format!("{:.1}k", damage as f64 / 1000.0)
    } else {
        damage.to_string()
    }
}

#[component]
pub fn StatsPage() -> impl IntoView {
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

    let has_team = Resource::new(
        || (),
        |_| async {
            crate::pages::team::dashboard::get_team_dashboard()
                .await
                .ok()
                .flatten()
                .is_some()
        },
    );
    let api_key = Resource::new(|| (), |_| check_api_key());
    let stats = Resource::new(|| (), |_| get_team_stats());
    let (sync_result, set_sync_result) = signal(Option::<Result<String, String>>::None);
    let (syncing, set_syncing) = signal(false);
    let (sync_queue, set_sync_queue) = signal(Option::<i32>::Some(440)); // default: Flex

    // Filters
    let (min_players, set_min_players) = signal(2_usize);
    let (filter_player, set_filter_player) = signal(String::new());
    let (filter_queue, set_filter_queue) = signal(0_i32); // 0 = all queues
    let (show_all, set_show_all) = signal(true); // true = show all matches by default

    let do_sync = move |_| {
        set_syncing.set(true);
        set_sync_result.set(None);
        let queue = sync_queue.get_untracked();
        leptos::task::spawn_local(async move {
            match sync_team_stats(queue).await {
                Ok(msg) => {
                    set_sync_result.set(Some(Ok(msg)));
                    stats.refetch();
                }
                Err(e) => set_sync_result.set(Some(Err(e.to_string()))),
            }
            set_syncing.set(false);
        });
    };

    view! {
        <div class="canvas-grain bg-base min-h-screen">
            <div class="max-w-7xl mx-auto px-8 lg:px-16 py-11 flex flex-col gap-6">
                // Imperial header
                <div class="flex items-end justify-between gap-6 flex-wrap">
                    <div class="flex flex-col gap-2">
                        <span class="font-imperial uppercase tracking-[0.18em] text-[10px] text-muted">
                            "Chapter VII · Battle log"
                        </span>
                        <BattleLogHeadline stats=stats />
                        <p class="font-display italic text-secondary text-base mt-1 max-w-[640px]">
                            "Each match is a chapter. Click a row to read the recap; the day's lesson sits in the right margin."
                        </p>
                    </div>
                    <div class="flex items-center gap-3">
                        // Queue type selector for sync
                        <select
                            class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2.5 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                let q = match v.as_str() {
                                    "420" => Some(420),
                                    "440" => Some(440),
                                    _ => None,
                                };
                                set_sync_queue.set(q);
                            }
                        >
                            <option value="440" selected>"Flex (440)"</option>
                            <option value="420">"Solo/Duo (420)"</option>
                            <option value="0">"All Queues"</option>
                        </select>
                        <button
                            class=move || if syncing.get() {
                                "bg-overlay-strong text-muted font-semibold rounded-lg px-5 py-2.5 text-sm cursor-not-allowed flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            } else {
                                "bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-5 py-2.5 text-sm transition-colors flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            }
                            on:click=do_sync
                            disabled=move || syncing.get()
                        >
                            {move || if syncing.get() {
                                view! {
                                    <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                    </svg>
                                    <span>"Syncing..."</span>
                                }.into_any()
                            } else {
                                view! { <span>"Sync Matches"</span> }.into_any()
                            }}
                        </button>
                    </div>
                </div>

                <HeraldicDivider />

                // API key warning
                <Suspense fallback=|| ()>
                    {move || api_key.get().map(|result| {
                        if let Ok(false) = result {
                            view! {
                                <div class="bg-warning/10 border border-warning/30 rounded-xl p-4 flex items-start gap-3" role="alert">
                                    <span class="text-warning text-lg flex-shrink-0" aria-hidden="true">"!"</span>
                                    <div>
                                        <p class="text-warning font-medium text-sm">"RIOT_API_KEY not configured"</p>
                                        <p class="text-warning/70 text-xs mt-1">"Add RIOT_API_KEY to your .env file to enable match history syncing. Get a key from developer.riotgames.com."</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    })}
                </Suspense>

                // Sync progress banner
                {move || if syncing.get() {
                    view! {
                        <div class="bg-info/10 border border-info/30 rounded-xl p-3 flex items-center gap-3" role="status" aria-live="polite">
                            <svg class="animate-spin h-4 w-4 text-info flex-shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                            </svg>
                            <span class="text-info text-sm">"Syncing matches from Riot API... This may take a moment."</span>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                // Sync result
                {move || sync_result.get().map(|r| match r {
                    Ok(msg) => view! {
                        <div class="bg-success/10 border border-success/30 rounded-xl p-3 text-success text-sm" role="status">{msg}</div>
                    }.into_any(),
                    Err(msg) => view! {
                        <div class="bg-danger/10 border border-danger/30 rounded-xl p-3 text-danger text-sm" role="alert">{msg}</div>
                    }.into_any(),
                })}

                // All / Team toggle (always visible)
                <div class="flex items-center gap-3">
                    <div class="flex rounded-lg overflow-hidden border border-outline/50">
                        <button
                            class=move || if show_all.get() {
                                "px-3 py-1.5 text-sm font-medium bg-accent text-accent-contrast focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            } else {
                                "px-3 py-1.5 text-sm font-medium bg-overlay/50 text-muted hover:text-primary transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            }
                            on:click=move |_| set_show_all.set(true)
                        >"All Matches"</button>
                        <button
                            class=move || if !show_all.get() {
                                "px-3 py-1.5 text-sm font-medium bg-accent text-accent-contrast focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            } else {
                                "px-3 py-1.5 text-sm font-medium bg-overlay/50 text-muted hover:text-primary transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                            }
                            on:click=move |_| set_show_all.set(false)
                        >"Team Games"</button>
                    </div>
                    <span class="text-dimmed text-xs">
                        {move || if show_all.get() {
                            "Showing all matches including solo queue"
                        } else {
                            "Showing games with 2+ team members"
                        }}
                    </span>
                </div>

                // Stats content
                <Suspense fallback=|| view! {
                    <div class="flex flex-col gap-2">
                        <SkeletonCard height="h-14" />
                        <SkeletonCard height="h-14" />
                        <SkeletonCard height="h-14" />
                        <SkeletonCard height="h-14" />
                        <SkeletonCard height="h-14" />
                    </div>
                }>
                    {move || stats.get().map(|result| match result {
                        Err(e) => view! {
                            <ErrorBanner message=format!("Failed to load stats: {e}") />
                        }.into_any(),
                        Ok(rows) if rows.is_empty() => {
                            let user_has_team = has_team.get().unwrap_or(false);
                            if user_has_team {
                                view! {
                                    <EmptyState
                                        icon="\u{2756}"
                                        message="No match stats yet — link your Riot account and play some games to see stats here"
                                        cta_label="Link Riot Account"
                                        cta_href="/profile"
                                    />
                                }.into_any()
                            } else {
                                view! { <NoTeamState /> }.into_any()
                            }
                        },
                        Ok(rows) => {
                            let all_matches = group_matches(&rows);
                            let unique_players: Vec<String> = {
                                let mut seen = HashSet::new();
                                rows.iter()
                                    .filter(|r| seen.insert(r.username.clone()))
                                    .map(|r| r.username.clone())
                                    .collect()
                            };
                            let roster_size = unique_players.len();

                            view! {
                                <StatsContent
                                    all_matches=all_matches
                                    unique_players=unique_players
                                    roster_size=roster_size
                                    min_players=min_players
                                    set_min_players=set_min_players
                                    filter_player=filter_player
                                    set_filter_player=set_filter_player
                                    filter_queue=filter_queue
                                    set_filter_queue=set_filter_queue
                                    show_all=show_all
                                />
                            }.into_any()
                        }
                    })}
                </Suspense>

                // Champion Trends (below match history)
                <ChampionTrendsSection />
            </div>
        </div>
    }
}

/// Imperial display headline: "A week of {wins} victories and {losses} defeats."
#[component]
fn BattleLogHeadline(
    stats: Resource<Result<Vec<TeamMatchRow>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <Suspense fallback=move || view! {
            <h1 class="font-display italic text-primary text-[40px] sm:text-[56px] leading-tight">
                "Battle log"
            </h1>
        }>
            {move || {
                let (wins, losses) = match stats.get() {
                    Some(Ok(rows)) => {
                        let groups = group_matches(&rows);
                        let w = groups.iter().filter(|g| g.win).count();
                        let l = groups.len() - w;
                        (w, l)
                    }
                    _ => (0_usize, 0_usize),
                };
                view! {
                    <h1 class="font-display italic text-primary text-[40px] sm:text-[56px] leading-tight">
                        "A week of "
                        <span class="text-success">{format!("{wins} victories")}</span>
                        " and "
                        <span class="text-danger">{format!("{losses} defeats")}</span>
                        "."
                    </h1>
                }.into_any()
            }}
        </Suspense>
    }
}

#[component]
fn StatsContent(
    all_matches: Vec<MatchGroup>,
    unique_players: Vec<String>,
    roster_size: usize,
    min_players: ReadSignal<usize>,
    set_min_players: WriteSignal<usize>,
    filter_player: ReadSignal<String>,
    set_filter_player: WriteSignal<String>,
    filter_queue: ReadSignal<i32>,
    set_filter_queue: WriteSignal<i32>,
    show_all: ReadSignal<bool>,
) -> impl IntoView {
    let all_matches = StoredValue::new(all_matches);
    let unique_players_for_filter = unique_players.clone();

    // Selected match for the right-rail folio recap.
    let selected_match: RwSignal<Option<String>> = RwSignal::new(None);

    let filtered = move || {
        let min = min_players.get();
        let player_filter = filter_player.get();
        let queue_filter = filter_queue.get();
        let all = show_all.get();

        all_matches.with_value(|matches| {
            matches
                .iter()
                .filter(|m| {
                    // Minimum players filter (skip when showing all)
                    if !all && m.players.len() < min {
                        return false;
                    }
                    // Player filter
                    if !player_filter.is_empty()
                        && !m.players.iter().any(|p| p.username == player_filter)
                    {
                        return false;
                    }
                    // Queue filter
                    if queue_filter != 0 && m.queue_id != queue_filter {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // Compute stats from filtered matches
    let computed_stats = move || {
        let matches = filtered();
        let total_games = matches.len();
        let wins = matches.iter().filter(|m| m.win).count();
        let wr = if total_games == 0 {
            "N/A".to_string()
        } else {
            format!("{:.0}%", wins as f64 / total_games as f64 * 100.0)
        };

        let all_players: Vec<&TeamMatchRow> =
            matches.iter().flat_map(|m| m.players.iter()).collect();
        let total_entries = all_players.len();
        let avg_kda = if total_entries == 0 {
            "N/A".to_string()
        } else {
            let k: f64 =
                all_players.iter().map(|p| p.kills as f64).sum::<f64>() / total_entries as f64;
            let d: f64 =
                all_players.iter().map(|p| p.deaths as f64).sum::<f64>() / total_entries as f64;
            let a: f64 =
                all_players.iter().map(|p| p.assists as f64).sum::<f64>() / total_entries as f64;
            format!("{k:.1}/{d:.1}/{a:.1}")
        };

        let avg_duration = if total_games == 0 {
            "N/A".to_string()
        } else {
            let total_secs: i32 = matches.iter().map(|m| m.game_duration).sum();
            format_duration(total_secs / total_games as i32)
        };

        (total_games.to_string(), wr, avg_kda, avg_duration)
    };

    // Auto-select the first match for folio recap when filtered results change.
    Effect::new(move |_| {
        let matches = filtered();
        let current = selected_match.get_untracked();
        let still_visible = current
            .as_ref()
            .map(|id| matches.iter().any(|m| &m.riot_match_id == id))
            .unwrap_or(false);
        if !still_visible {
            selected_match.set(matches.first().map(|m| m.riot_match_id.clone()));
        }
    });

    view! {
        <div class="flex flex-col gap-6">
            // Filters
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex items-center gap-4 flex-wrap">
                <span class="font-imperial uppercase tracking-wider text-xs text-muted">"Filters"</span>

                // Queue type filter
                <div class="flex items-center gap-2">
                    <span class="text-secondary text-sm">"Queue:"</span>
                    <select
                        class="bg-overlay/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                        on:change=move |ev| {
                            let v: i32 = event_target_value(&ev).parse().unwrap_or(0);
                            set_filter_queue.set(v);
                        }
                    >
                        <option value="0">"All Queues"</option>
                        <option value="420">"Solo/Duo"</option>
                        <option value="440">"Flex"</option>
                        <option value="450">"ARAM"</option>
                    </select>
                </div>

                // Min players dropdown (hidden when showing all matches)
                {move || if !show_all.get() {
                    view! {
                        <span class="text-overlay-strong" aria-hidden="true">"|"</span>
                        <div class="flex items-center gap-2">
                            <span class="text-secondary text-sm">"Min. players:"</span>
                            <select
                                class="bg-overlay/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                on:change=move |ev| {
                                    let v: usize = event_target_value(&ev).parse().unwrap_or(2);
                                    set_min_players.set(v);
                                }
                            >
                                {(2..=roster_size.max(2)).map(|n| {
                                    let label = if n == roster_size && roster_size >= 5 {
                                        format!("{n} (full roster)")
                                    } else {
                                        n.to_string()
                                    };
                                    view! { <option value=n.to_string()>{label}</option> }
                                }).collect_view()}
                            </select>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                <span class="text-overlay-strong" aria-hidden="true">"|"</span>

                // Player filter
                <select
                    class="bg-overlay/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                    on:change=move |ev| set_filter_player.set(event_target_value(&ev))
                >
                    <option value="">"All Players"</option>
                    {unique_players_for_filter.into_iter().map(|name| {
                        let name_val = name.clone();
                        view! { <option value=name_val>{name}</option> }
                    }).collect_view()}
                </select>

                <span class="text-dimmed text-xs ml-auto tabular-nums">
                    {move || {
                        let (total, _, _, _) = computed_stats();
                        format!("{total} matches")
                    }}
                </span>
            </div>

            // Summary cards
            {move || {
                let (total, wr, kda, duration) = computed_stats();
                view! {
                    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                        <StatCard label="Games".to_string() value=total />
                        <StatCard label="Win Rate".to_string() value=wr />
                        <StatCard label="Avg KDA".to_string() value=kda />
                        <StatCard label="Avg Duration".to_string() value=duration />
                    </div>
                }
            }}

            // Two-column layout: 1.4fr battle log + 1fr folio recap
            <div class="grid grid-cols-1 lg:grid-cols-[1.4fr_1fr] gap-8 items-start">
                // Battle log column
                <div class="bg-elevated border border-divider rounded-xl p-6">
                    <div class="flex items-baseline justify-between mb-4">
                        <span class="font-imperial uppercase tracking-wider text-xs text-muted">
                            "Battle log"
                        </span>
                        <span class="font-mono text-xs text-dimmed tabular-nums">
                            {move || {
                                let (total, wr, _, _) = computed_stats();
                                format!("{total} matches · {wr}")
                            }}
                        </span>
                    </div>
                    <div class="flex flex-col">
                        {move || {
                            let matches = filtered();
                            if matches.is_empty() {
                                return view! {
                                    <p class="text-dimmed text-sm py-8 text-center">"No matches match current filters."</p>
                                }.into_any();
                            }
                            let active = selected_match.get();
                            view! {
                                <div class="flex flex-col">
                                    {matches.into_iter().enumerate().map(|(idx, m)| {
                                        let riot_id = m.riot_match_id.clone();
                                        let riot_id_click = riot_id.clone();
                                        let is_active = active.as_ref() == Some(&riot_id);
                                        let win = m.win;
                                        let date = format_date(&m.game_end);
                                        let duration = format_duration(m.game_duration);
                                        let player_count = m.players.len();
                                        let q_label = queue_label(m.queue_id);

                                        // Use the primary (first) player for the row summary
                                        let first_player = m.players.first().cloned();

                                        // Aggregate team KDA
                                        let total_kills: i32 = m.players.iter().map(|p| p.kills).sum();
                                        let total_deaths: i32 = m.players.iter().map(|p| p.deaths).sum();
                                        let total_assists: i32 = m.players.iter().map(|p| p.assists).sum();

                                        // 6-cell grid: 8px result bar | 56px champ tile | 1fr meta | 130px KDA | 80px dur | 24px chevron
                                        let row_class = if is_active {
                                            "grid grid-cols-[8px_56px_1fr_130px_80px_24px] gap-4 items-center py-4 px-3 -mx-3 border-b border-divider/30 last:border-b-0 bg-accent-soft cursor-pointer transition-all duration-200 text-left rounded-md focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                        } else {
                                            "grid grid-cols-[8px_56px_1fr_130px_80px_24px] gap-4 items-center py-4 px-3 -mx-3 border-b border-divider/30 last:border-b-0 hover:bg-overlay/30 cursor-pointer transition-all duration-200 text-left focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                        };
                                        let bar_class = if win { "bg-success self-stretch rounded-sm" } else { "bg-danger self-stretch rounded-sm" };
                                        let result_word = if win { "Victory" } else { "Defeat" };
                                        let result_color = if win { "text-success" } else { "text-danger" };
                                        let chevron = if is_active { "\u{25BC}" } else { "\u{25B6}" };
                                        let _ = idx;

                                        view! {
                                            <button
                                                type="button"
                                                class=row_class
                                                on:click=move |_| {
                                                    let cur = selected_match.get_untracked();
                                                    if cur.as_ref() == Some(&riot_id_click) {
                                                        // Re-clicking the active row keeps it open (toggle off would hide the recap)
                                                        return;
                                                    }
                                                    selected_match.set(Some(riot_id_click.clone()));
                                                }
                                            >
                                                // Cell 1: result bar
                                                <div class=bar_class aria-hidden="true"></div>

                                                // Cell 2: champion tile (56px)
                                                {first_player.as_ref().map(|fp| {
                                                    let icon_url = champion_icon_url(&fp.champion);
                                                    let champ = fp.champion.clone();
                                                    view! {
                                                        <img
                                                            src=icon_url
                                                            alt=champ
                                                            class="w-12 h-12 rounded-lg border border-divider/50 object-cover"
                                                        />
                                                    }
                                                })}

                                                // Cell 3: meta (champion name + queue + duration + result word)
                                                {first_player.as_ref().map(|fp| {
                                                    let champ = fp.champion.clone();
                                                    let username = fp.username.clone();
                                                    view! {
                                                        <div class="min-w-0">
                                                            <div class="flex items-baseline gap-2 flex-wrap">
                                                                <span class=format!("font-imperial uppercase tracking-wider text-[10px] {result_color}")>{result_word}</span>
                                                                <span class="font-mono text-[11px] text-dimmed tabular-nums">{duration.clone()}</span>
                                                                <span class="font-mono text-[11px] text-dimmed">"·"</span>
                                                                <span class="font-mono text-[11px] text-dimmed">{q_label}</span>
                                                                {if player_count > 1 {
                                                                    view! {
                                                                        <span class="font-mono text-[11px] text-dimmed">
                                                                            {format!("· {player_count}p")}
                                                                        </span>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                            <div class="font-display italic text-secondary text-sm mt-1 truncate">
                                                                {format!("\"{champ} · {username}\"")}
                                                            </div>
                                                        </div>
                                                    }
                                                })}

                                                // Cell 4: KDA (130px)
                                                {first_player.as_ref().map(|fp| {
                                                    let kda_str = format!("{}/{}/{}", fp.kills, fp.deaths, fp.assists);
                                                    let team_str = if player_count > 1 {
                                                        Some(format!("team {total_kills}/{total_deaths}/{total_assists}"))
                                                    } else {
                                                        None
                                                    };
                                                    view! {
                                                        <div class="text-right">
                                                            <div class="font-mono text-sm text-primary tabular-nums">{kda_str}</div>
                                                            <div class="font-imperial uppercase tracking-wider text-[9px] text-muted mt-0.5">"K/D/A"</div>
                                                            {team_str.map(|s| view! {
                                                                <div class="font-mono text-[10px] text-dimmed mt-0.5 tabular-nums">{s}</div>
                                                            })}
                                                        </div>
                                                    }
                                                })}

                                                // Cell 5: damage (80px)
                                                {first_player.as_ref().map(|fp| {
                                                    let dmg = format_damage(fp.damage);
                                                    view! {
                                                        <div class="text-right">
                                                            <div class="font-mono text-sm text-secondary tabular-nums">{dmg}</div>
                                                            <div class="text-dimmed text-[10px] mt-0.5">"dmg · " {date.clone()}</div>
                                                        </div>
                                                    }
                                                })}

                                                // Cell 6: chevron (24px)
                                                <span class=move || if is_active {
                                                    "text-accent text-xs text-center"
                                                } else {
                                                    "text-dimmed text-xs text-center"
                                                } aria-hidden="true">{chevron}</span>
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>

                // Folio recap column (right rail, sticky)
                <div class="lg:sticky lg:top-24 self-start flex flex-col gap-4">
                    {move || {
                        let matches = filtered();
                        let active_id = selected_match.get();
                        let active_match = active_id
                            .as_ref()
                            .and_then(|id| matches.iter().find(|m| &m.riot_match_id == id).cloned());

                        match active_match {
                            None => view! {
                                <div class="bg-elevated border border-divider rounded-xl p-6 text-center">
                                    <span class="font-imperial uppercase tracking-wider text-[10px] text-muted">"Folio recap"</span>
                                    <p class="text-dimmed text-sm mt-3">"Select a match to read the recap."</p>
                                </div>
                            }.into_any(),
                            Some(m) => {
                                let win = m.win;
                                let result_word = if win { "Victory" } else { "Defeat" };
                                let result_color = if win { "text-success" } else { "text-danger" };
                                let primary = m.players.first().cloned();
                                let players_for_detail = m.players.clone();
                                let q_label = queue_label(m.queue_id);
                                let duration = format_duration(m.game_duration);
                                let date = format_date(&m.game_end);
                                let player_count = m.players.len();
                                let total_kills: i32 = m.players.iter().map(|p| p.kills).sum();
                                let total_deaths: i32 = m.players.iter().map(|p| p.deaths).sum();
                                let total_assists: i32 = m.players.iter().map(|p| p.assists).sum();
                                let detail_href = format!("/match/{}", m.riot_match_id);

                                view! {
                                    <div class="bg-elevated border border-divider rounded-xl p-6 transition-all duration-200">
                                        <div class="flex items-baseline justify-between mb-4">
                                            <span class=format!("font-imperial uppercase tracking-wider text-[10px] {result_color}")>
                                                {format!("{result_word} · folio")}
                                            </span>
                                            <span class="font-mono text-[11px] text-dimmed truncate max-w-[140px]">
                                                {format!("match {}", m.riot_match_id)}
                                            </span>
                                        </div>

                                        // Hero: champ tile + name + meta
                                        {primary.as_ref().map(|fp| {
                                            let icon_url = champion_icon_url(&fp.champion);
                                            let champ = fp.champion.clone();
                                            let username = fp.username.clone();
                                            view! {
                                                <div class="flex items-center gap-4 mb-4">
                                                    <img src=icon_url alt=champ.clone() class="w-16 h-16 rounded-lg border border-outline/50 object-cover" />
                                                    <div class="min-w-0">
                                                        <div class="font-display italic text-primary text-2xl leading-tight truncate">
                                                            {champ}
                                                        </div>
                                                        <div class="font-mono text-xs text-secondary mt-1 truncate">
                                                            {format!("{username} · {q_label} · {duration}")}
                                                        </div>
                                                        <div class="font-mono text-[11px] text-dimmed mt-0.5">{date.clone()}</div>
                                                    </div>
                                                </div>
                                            }
                                        })}

                                        // Folio stats
                                        {primary.as_ref().map(|fp| {
                                            let kda_str = format!("{}/{}/{}", fp.kills, fp.deaths, fp.assists);
                                            let cs_str = fp.cs.to_string();
                                            let dmg_str = format_damage(fp.damage);
                                            let vis_str = fp.vision_score.to_string();
                                            view! {
                                                <div class="grid grid-cols-3 gap-2 mb-4">
                                                    <FolioStat label="K/D/A".to_string() value=kda_str />
                                                    <FolioStat label="CS".to_string() value=cs_str />
                                                    <FolioStat label="Damage".to_string() value=dmg_str />
                                                    <FolioStat label="Vision".to_string() value=vis_str />
                                                    <FolioStat label="Players".to_string() value=player_count.to_string() />
                                                    <FolioStatTone
                                                        label="Result".to_string()
                                                        value={if win { "Won".to_string() } else { "Lost".to_string() }}
                                                        tone={if win { "text-success" } else { "text-danger" }}
                                                    />
                                                </div>
                                            }
                                        })}

                                        // Team note when multiple roster members played
                                        {if player_count > 1 {
                                            view! {
                                                <div class="bg-accent-soft border-l-[3px] border-accent rounded-r p-3 mb-4">
                                                    <span class="font-imperial uppercase tracking-wider text-[9px] text-accent">"Team line"</span>
                                                    <div class="font-display italic text-primary text-sm mt-1">
                                                        {format!("{player_count} roster members · combined {total_kills}/{total_deaths}/{total_assists}")}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}

                                        // CTA
                                        <a
                                            href=detail_href
                                            class="inline-flex items-center gap-2 bg-accent hover:bg-accent-hover text-accent-contrast font-semibold px-4 py-2 rounded-lg text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                        >
                                            "Open match detail"
                                        </a>
                                    </div>

                                    // Companion card: full team scoreboard for this match
                                    {if player_count > 1 {
                                        view! {
                                            <div class="bg-elevated border border-divider rounded-xl p-6">
                                                <span class="font-imperial uppercase tracking-wider text-[10px] text-muted">"Roster line"</span>
                                                <table class="w-full mt-3 text-sm">
                                                    <thead>
                                                        <tr class="border-b border-divider/30 text-muted text-[10px] uppercase tracking-wider">
                                                            <th class="text-left py-1.5 font-normal">"Player"</th>
                                                            <th class="text-left py-1.5 font-normal">"Champion"</th>
                                                            <th class="text-right py-1.5 font-normal">"K/D/A"</th>
                                                            <th class="text-right py-1.5 font-normal">"Result"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {players_for_detail.into_iter().map(|p| {
                                                            let icon_url = champion_icon_url(&p.champion);
                                                            let kda_str = format!("{}/{}/{}", p.kills, p.deaths, p.assists);
                                                            let p_result_cls = if p.win { "text-success" } else { "text-danger" };
                                                            let p_result = if p.win { "Won" } else { "Lost" };
                                                            view! {
                                                                <tr class="border-b border-divider/20 last:border-0">
                                                                    <td class="py-2 text-primary text-sm">{p.username}</td>
                                                                    <td class="py-2">
                                                                        <div class="flex items-center gap-2">
                                                                            <img src=icon_url alt=p.champion.clone() class="w-6 h-6 rounded" />
                                                                            <span class="text-secondary text-sm">{p.champion}</span>
                                                                        </div>
                                                                    </td>
                                                                    <td class="py-2 text-right font-mono text-sm text-primary tabular-nums">{kda_str}</td>
                                                                    <td class=format!("py-2 text-right font-imperial uppercase tracking-wider text-[10px] {p_result_cls}")>{p_result}</td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn FolioStat(label: String, value: String) -> impl IntoView {
    view! {
        <div class="bg-surface border border-outline/50 rounded-md px-3 py-2.5">
            <span class="font-imperial uppercase tracking-wider text-[9px] text-muted">{label}</span>
            <div class="font-mono text-base text-primary mt-1 tabular-nums">{value}</div>
        </div>
    }
}

#[component]
fn FolioStatTone(label: String, value: String, tone: &'static str) -> impl IntoView {
    let value_class = format!("font-mono text-base mt-1 tabular-nums {tone}");
    view! {
        <div class="bg-surface border border-outline/50 rounded-md px-3 py-2.5">
            <span class="font-imperial uppercase tracking-wider text-[9px] text-muted">{label}</span>
            <div class=value_class>{value}</div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Champion Trends section (Phase 15 / LEARN-06)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum TrendSortColumn {
    Champion,
    Games,
    WinPct,
    Kda,
    CsPerMin,
    AvgDamage,
}

#[component]
fn ChampionTrendsSection() -> impl IntoView {
    let trends_window: RwSignal<&'static str> = RwSignal::new("30d");
    let trends_resource = Resource::new(
        move || trends_window.get(),
        |w| async move { get_champion_trends(w.to_string()).await },
    );

    let sort_col: RwSignal<TrendSortColumn> = RwSignal::new(TrendSortColumn::Games);
    let sort_dir_desc: RwSignal<bool> = RwSignal::new(true);
    let show_all: RwSignal<bool> = RwSignal::new(false);

    let sorted_trends = Memo::new(move |_| -> Vec<ChampionTrend> {
        let data = match trends_resource.get() {
            Some(Ok(d)) => d,
            _ => return Vec::new(),
        };
        let min_games = if show_all.get() { 0 } else { 3 };
        let mut filtered: Vec<ChampionTrend> =
            data.into_iter().filter(|t| t.games >= min_games).collect();
        let col = sort_col.get();
        let desc = sort_dir_desc.get();
        filtered.sort_by(|a, b| {
            let ord = match col {
                TrendSortColumn::Champion => a.champion.cmp(&b.champion),
                TrendSortColumn::Games => a.games.cmp(&b.games),
                TrendSortColumn::WinPct => {
                    let aw = a.wins as f32 / a.games.max(1) as f32;
                    let bw = b.wins as f32 / b.games.max(1) as f32;
                    aw.partial_cmp(&bw).unwrap_or(std::cmp::Ordering::Equal)
                }
                TrendSortColumn::Kda => a
                    .avg_kda
                    .partial_cmp(&b.avg_kda)
                    .unwrap_or(std::cmp::Ordering::Equal),
                TrendSortColumn::CsPerMin => a
                    .cs_per_min
                    .partial_cmp(&b.cs_per_min)
                    .unwrap_or(std::cmp::Ordering::Equal),
                TrendSortColumn::AvgDamage => a.avg_damage.cmp(&b.avg_damage),
            };
            if desc {
                ord.reverse()
            } else {
                ord
            }
        });
        filtered
    });

    let on_header_click = move |target: TrendSortColumn| {
        if sort_col.get() == target {
            sort_dir_desc.update(|d| *d = !*d);
        } else {
            sort_col.set(target);
            sort_dir_desc.set(true);
        }
    };

    let render_pill = move |w: &'static str| {
        let active = move || trends_window.get() == w;
        view! {
            <button
                class=move || if active() {
                    "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                } else {
                    "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                }
                on:click=move |_| trends_window.set(w)
            >{w}</button>
        }
    };

    let header_cell = move |label: &'static str, target: TrendSortColumn| {
        let is_active = move || sort_col.get() == target;
        let arrow = move || {
            if is_active() {
                if sort_dir_desc.get() { " \u{25BE}" } else { " \u{25B4}" }
            } else {
                " \u{25BE}"
            }
        };
        view! {
            <th
                class=move || format!(
                    "px-3 py-2 text-left text-xs font-normal uppercase tracking-wider cursor-pointer select-none transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none {}",
                    if is_active() { "text-secondary" } else { "text-muted hover:text-secondary opacity-70" }
                )
                on:click=move |_| on_header_click(target)
            >
                <span>{label}</span><span class=move || if is_active() { "" } else { " opacity-30" }>{arrow}</span>
            </th>
        }
    };

    view! {
        <div class="bg-surface border border-divider rounded-xl p-6 flex flex-col gap-4 mt-2">
            <div class="flex items-center justify-between">
                <div class="flex flex-col gap-1">
                    <span class="font-imperial uppercase tracking-wider text-xs text-muted">"Champion trends"</span>
                    <h2 class="font-display italic text-primary text-2xl">"The week's pattern"</h2>
                </div>
                <div class="flex items-center gap-2">
                    {render_pill("7d")}
                    {render_pill("30d")}
                    {render_pill("90d")}
                    {render_pill("All-time")}
                </div>
            </div>

            <Suspense fallback=|| view! { <SkeletonCard height="h-48" /> }>
                {move || trends_resource.get().map(|result| match result {
                    Err(_) => view! {
                        <ErrorBanner message="Could not load champion trends. Refresh to try again.".to_string() />
                    }.into_any(),
                    Ok(rows) if rows.is_empty() => view! {
                        <EmptyState message="Sync your match history to see champion trends." />
                    }.into_any(),
                    Ok(_) => view! {
                        <div class="flex flex-col gap-2">
                            <div class="overflow-x-auto">
                                <table class="w-full">
                                    <thead class="bg-elevated/50">
                                        <tr>
                                            {header_cell("Champion", TrendSortColumn::Champion)}
                                            {header_cell("Games", TrendSortColumn::Games)}
                                            {header_cell("Win %", TrendSortColumn::WinPct)}
                                            {header_cell("KDA", TrendSortColumn::Kda)}
                                            {header_cell("CS/min", TrendSortColumn::CsPerMin)}
                                            {header_cell("Avg Damage", TrendSortColumn::AvgDamage)}
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || {
                                            let visible = sorted_trends.get();
                                            if visible.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="6" class="py-6 text-center text-muted text-sm">
                                                            "All champions hidden by min-games filter. "
                                                            <button class="text-accent hover:text-accent-hover text-sm cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded"
                                                                    on:click=move |_| show_all.set(true)>
                                                                "Show all"
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <For
                                                        each=move || sorted_trends.get()
                                                        key=|t: &ChampionTrend| t.champion.clone()
                                                        children=move |t: ChampionTrend| view! {
                                                            <ChampionTrendRow trend=t />
                                                        }
                                                    />
                                                }.into_any()
                                            }
                                        }}
                                    </tbody>
                                </table>
                            </div>
                            <div class="flex justify-end">
                                <button class="text-accent hover:text-accent-hover text-xs font-normal transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded"
                                        on:click=move |_| show_all.update(|v| *v = !*v)>
                                    {move || if show_all.get() {
                                        "Hide low sample (< 3 games)"
                                    } else {
                                        "Show all champions"
                                    }}
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn ChampionTrendRow(trend: ChampionTrend) -> impl IntoView {
    let icon_errored = RwSignal::new(false);
    let win_pct = (trend.wins as f32 / trend.games.max(1) as f32 * 100.0).round();
    let champion_for_icon = trend.champion.clone();
    let champion_for_label = trend.champion.clone();
    view! {
        <tr class="border-t border-divider/30 hover:bg-elevated/30 transition-colors h-11">
            <td class="px-3 py-2.5 text-sm font-semibold text-primary">
                <div class="flex items-center gap-2">
                    {move || if icon_errored.get() {
                        view! { <div class="w-5 h-5 rounded bg-elevated border border-divider/30" /> }.into_any()
                    } else {
                        let icon_src = champion_icon_url(&champion_for_icon);
                        view! {
                            <img src=icon_src class="w-5 h-5 rounded object-contain"
                                 on:error=move |_| icon_errored.set(true) />
                        }.into_any()
                    }}
                    <span>{champion_for_label}</span>
                </div>
            </td>
            <td class="px-3 py-2.5 text-center text-sm text-secondary tabular-nums">{trend.games}</td>
            <td class="px-3 py-2.5 text-center text-sm text-secondary tabular-nums">{format!("{:.1}%", win_pct)}</td>
            <td class="px-3 py-2.5 text-center text-sm font-semibold text-primary tabular-nums">{format!("{:.1}", trend.avg_kda)}</td>
            <td class="px-3 py-2.5 text-center text-sm text-secondary tabular-nums">{format!("{:.1}", trend.cs_per_min)}</td>
            <td class="px-3 py-2.5 pr-3 text-right text-sm text-secondary tabular-nums">{format_damage(trend.avg_damage)}</td>
        </tr>
    }
}
