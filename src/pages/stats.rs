use crate::components::stat_card::StatCard;
use crate::components::ui::ErrorBanner;
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

        match riot::fetch_match_history(puuid, queue_id).await {
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
        <div class="max-w-6xl mx-auto py-8 px-6 flex flex-col gap-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-primary">"Team Stats"</h1>
                    <p class="text-muted text-sm mt-1">"Match history synced from the Riot API"</p>
                </div>
                <div class="flex items-center gap-3">
                    // Queue type selector for sync
                    <select
                        class="bg-surface/50 border border-outline/50 rounded-lg px-3 py-2.5 text-primary text-sm focus:outline-none focus:border-accent/50"
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
                            "bg-overlay-strong text-muted font-semibold rounded-lg px-5 py-2.5 text-sm cursor-not-allowed flex items-center gap-2"
                        } else {
                            "bg-blue-500 hover:bg-blue-400 text-white font-semibold rounded-lg px-5 py-2.5 text-sm transition-colors flex items-center gap-2"
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

            // API key warning
            <Suspense fallback=|| ()>
                {move || api_key.get().map(|result| {
                    if let Ok(false) = result {
                        view! {
                            <div class="bg-amber-500/10 border border-amber-500/30 rounded-xl p-4 flex items-start gap-3">
                                <span class="text-amber-400 text-lg flex-shrink-0">"!"</span>
                                <div>
                                    <p class="text-amber-400 font-medium text-sm">"RIOT_API_KEY not configured"</p>
                                    <p class="text-amber-400/70 text-xs mt-1">"Add RIOT_API_KEY to your .env file to enable match history syncing. Get a key from developer.riotgames.com."</p>
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
                    <div class="bg-blue-500/10 border border-blue-500/30 rounded-xl p-3 flex items-center gap-3">
                        <svg class="animate-spin h-4 w-4 text-blue-400 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                        </svg>
                        <span class="text-blue-400 text-sm">"Syncing matches from Riot API... This may take a moment."</span>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            // Sync result
            {move || sync_result.get().map(|r| match r {
                Ok(msg) => view! {
                    <div class="bg-emerald-500/10 border border-emerald-500/30 rounded-xl p-3 text-emerald-400 text-sm">{msg}</div>
                }.into_any(),
                Err(msg) => view! {
                    <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-3 text-red-400 text-sm">{msg}</div>
                }.into_any(),
            })}

            // All / Team toggle (always visible)
            <div class="flex items-center gap-3">
                <div class="flex rounded-lg overflow-hidden border border-outline/50">
                    <button
                        class=move || if show_all.get() {
                            "px-3 py-1.5 text-sm font-medium bg-accent text-accent-contrast"
                        } else {
                            "px-3 py-1.5 text-sm font-medium bg-overlay/50 text-muted hover:text-primary transition-colors"
                        }
                        on:click=move |_| set_show_all.set(true)
                    >"All Matches"</button>
                    <button
                        class=move || if !show_all.get() {
                            "px-3 py-1.5 text-sm font-medium bg-accent text-accent-contrast"
                        } else {
                            "px-3 py-1.5 text-sm font-medium bg-overlay/50 text-muted hover:text-primary transition-colors"
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
            <Suspense fallback=|| view! { <div class="text-dimmed text-center py-8">"Loading stats..."</div> }>
                {move || stats.get().map(|result| match result {
                    Err(e) => view! {
                        <ErrorBanner message=format!("Failed to load stats: {e}") />
                    }.into_any(),
                    Ok(rows) if rows.is_empty() => view! {
                        <div class="text-center py-12">
                            <p class="text-muted text-lg mb-2">"No match data yet"</p>
                            <p class="text-dimmed text-sm mb-6">"Create or join a team, then click Sync Matches to pull recent games."</p>
                            <a href="/team/roster" class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-2 transition-colors">
                                "Go to Team"
                            </a>
                        </div>
                    }.into_any(),
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
        </div>
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

    // Expanded match detail (Task 3)
    let expanded_match: RwSignal<Option<String>> = RwSignal::new(None);

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

    view! {
        <div class="flex flex-col gap-6">
            // Filters
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex items-center gap-4 flex-wrap">
                <span class="text-muted text-sm font-medium">"Filters:"</span>

                // Queue type filter
                <div class="flex items-center gap-2">
                    <span class="text-secondary text-sm">"Queue:"</span>
                    <select
                        class="bg-overlay/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm focus:outline-none focus:border-accent/50"
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
                        <span class="text-overlay-strong">"|"</span>
                        <div class="flex items-center gap-2">
                            <span class="text-secondary text-sm">"Min. players:"</span>
                            <select
                                class="bg-overlay/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm focus:outline-none focus:border-accent/50"
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

                <span class="text-overlay-strong">"|"</span>

                // Player filter
                <select
                    class="bg-overlay/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm focus:outline-none focus:border-accent/50"
                    on:change=move |ev| set_filter_player.set(event_target_value(&ev))
                >
                    <option value="">"All Players"</option>
                    {unique_players_for_filter.into_iter().map(|name| {
                        let name_val = name.clone();
                        view! { <option value=name_val>{name}</option> }
                    }).collect_view()}
                </select>

                <span class="text-dimmed text-xs ml-auto">
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
                    <div class="grid grid-cols-4 gap-4">
                        <StatCard label="Games".to_string() value=total />
                        <StatCard label="Win Rate".to_string() value=wr />
                        <StatCard label="Avg KDA".to_string() value=kda />
                        <StatCard label="Avg Duration".to_string() value=duration />
                    </div>
                }
            }}

            // Match list (Task 2: OP.GG-style layout)
            <div>
                <h2 class="text-primary font-semibold text-lg mb-3">"Match History"</h2>
                <div class="flex flex-col gap-1.5">
                    {move || {
                        let matches = filtered();
                        if matches.is_empty() {
                            return view! {
                                <p class="text-dimmed text-sm py-4 text-center">"No matches match current filters."</p>
                            }.into_any();
                        }
                        let expanded = expanded_match.get();
                        view! {
                            <div class="flex flex-col gap-1.5">
                                {matches.into_iter().map(|m| {
                                    let riot_id = m.riot_match_id.clone();
                                    let riot_id_click = riot_id.clone();
                                    let riot_id_expand = riot_id.clone();
                                    let is_expanded = expanded.as_ref() == Some(&riot_id);
                                    let win = m.win;
                                    let date = format_date(&m.game_end);
                                    let duration = format_duration(m.game_duration);
                                    let player_count = m.players.len();
                                    let q_label = queue_label(m.queue_id);

                                    // For the summary row, show the "primary" player
                                    // (first player alphabetically, or just the first)
                                    let first_player = m.players.first().cloned();
                                    let players_for_detail = m.players.clone();

                                    // Aggregate team KDA for the match
                                    let total_kills: i32 = m.players.iter().map(|p| p.kills).sum();
                                    let total_deaths: i32 = m.players.iter().map(|p| p.deaths).sum();
                                    let total_assists: i32 = m.players.iter().map(|p| p.assists).sum();

                                    // Background tint based on win/loss
                                    let row_bg = if win {
                                        "bg-blue-900/20 border border-blue-500/20 hover:bg-blue-900/30"
                                    } else {
                                        "bg-red-900/20 border border-red-500/20 hover:bg-red-900/30"
                                    };

                                    let result_badge_cls = if win {
                                        "bg-blue-500/20 text-blue-400 text-xs font-bold px-2 py-0.5 rounded"
                                    } else {
                                        "bg-red-500/20 text-red-400 text-xs font-bold px-2 py-0.5 rounded"
                                    };
                                    let result_text = if win { "WIN" } else { "LOSS" };

                                    view! {
                                        <div>
                                            // Match summary row
                                            <div
                                                class=format!("{row_bg} rounded-lg px-4 py-3 cursor-pointer transition-colors")
                                                on:click=move |_| {
                                                    let current = expanded_match.get_untracked();
                                                    if current.as_ref() == Some(&riot_id_click) {
                                                        expanded_match.set(None);
                                                    } else {
                                                        expanded_match.set(Some(riot_id_click.clone()));
                                                    }
                                                }
                                            >
                                                <div class="flex items-center gap-4">
                                                    // Left: Champion icon + name (from first player)
                                                    {first_player.map(|fp| {
                                                        let icon_url = champion_icon_url(&fp.champion);
                                                        let champ_name = fp.champion.clone();
                                                        let kda_str = format!("{}/{}/{}", fp.kills, fp.deaths, fp.assists);
                                                        let kda_ratio = if fp.deaths == 0 {
                                                            "Perfect".to_string()
                                                        } else {
                                                            format!("{:.2}", (fp.kills + fp.assists) as f64 / fp.deaths as f64)
                                                        };
                                                        view! {
                                                            <div class="flex items-center gap-3 min-w-[200px]">
                                                                <img
                                                                    src=icon_url
                                                                    alt=champ_name.clone()
                                                                    class="w-10 h-10 rounded-lg"
                                                                />
                                                                <div>
                                                                    <div class="text-primary text-sm font-medium">{champ_name}</div>
                                                                    <div class="text-secondary text-xs">{fp.username}</div>
                                                                </div>
                                                            </div>

                                                            // KDA
                                                            <div class="min-w-[100px] text-center">
                                                                <div class="text-primary text-sm font-semibold">{kda_str}</div>
                                                                <div class="text-muted text-xs">{format!("{kda_ratio} KDA")}</div>
                                                            </div>

                                                            // CS + Vision
                                                            <div class="min-w-[80px] text-center">
                                                                <div class="text-secondary text-sm">{format!("{} CS", fp.cs)}</div>
                                                                <div class="text-dimmed text-xs">{format!("{} vis", fp.vision_score)}</div>
                                                            </div>

                                                            // Damage
                                                            <div class="min-w-[70px] text-center">
                                                                <div class="text-secondary text-sm">{format_damage(fp.damage)}</div>
                                                                <div class="text-dimmed text-xs">"dmg"</div>
                                                            </div>
                                                        }
                                                    })}

                                                    // Team KDA (if multiple players)
                                                    {if player_count > 1 {
                                                        view! {
                                                            <div class="min-w-[80px] text-center border-l border-divider/30 pl-4">
                                                                <div class="text-secondary text-xs">"Team"</div>
                                                                <div class="text-primary text-sm font-medium">{format!("{total_kills}/{total_deaths}/{total_assists}")}</div>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }}

                                                    // Right: Win/Loss badge, duration, date, queue
                                                    <div class="ml-auto flex items-center gap-3 text-right">
                                                        <div>
                                                            <span class=result_badge_cls>{result_text}</span>
                                                        </div>
                                                        <div class="min-w-[60px]">
                                                            <div class="text-secondary text-sm">{duration}</div>
                                                            <div class="text-dimmed text-xs">{date}</div>
                                                        </div>
                                                        <div class="min-w-[60px]">
                                                            <div class="text-muted text-xs">{q_label}</div>
                                                            <div class={
                                                                if player_count >= 5 {
                                                                    "text-emerald-400/70 text-xs font-medium"
                                                                } else {
                                                                    "text-dimmed text-xs"
                                                                }
                                                            }>
                                                                {format!("{player_count}p")}
                                                            </div>
                                                        </div>
                                                        // Expand indicator
                                                        <span class="text-dimmed text-xs">
                                                            {if is_expanded { "^" } else { "v" }}
                                                        </span>
                                                    </div>
                                                </div>
                                            </div>

                                            // Task 3: Expandable match detail panel
                                            {if is_expanded {
                                                let detail_bg = if win {
                                                    "bg-blue-950/30 border border-blue-500/10"
                                                } else {
                                                    "bg-red-950/30 border border-red-500/10"
                                                };
                                                view! {
                                                    <div class=format!("{detail_bg} rounded-b-lg px-4 py-3 -mt-0.5")>
                                                        <div class="text-muted text-xs font-medium mb-2 flex items-center justify-between">
                                                            <span>{format!("Match ID: {}", riot_id_expand)}</span>
                                                            <span>{format!("{player_count} team members in this game")}</span>
                                                        </div>
                                                        // Scoreboard table
                                                        <table class="w-full text-sm">
                                                            <thead>
                                                                <tr class="text-muted text-xs border-b border-divider/30">
                                                                    <th class="text-left py-1.5 font-medium">"Player"</th>
                                                                    <th class="text-left py-1.5 font-medium">"Champion"</th>
                                                                    <th class="text-center py-1.5 font-medium">"KDA"</th>
                                                                    <th class="text-center py-1.5 font-medium">"CS"</th>
                                                                    <th class="text-center py-1.5 font-medium">"Vision"</th>
                                                                    <th class="text-center py-1.5 font-medium">"Damage"</th>
                                                                    <th class="text-center py-1.5 font-medium">"Result"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                {players_for_detail.into_iter().map(|p| {
                                                                    let icon_url = champion_icon_url(&p.champion);
                                                                    let kda_str = format!("{}/{}/{}", p.kills, p.deaths, p.assists);
                                                                    let kda_ratio = if p.deaths == 0 {
                                                                        "Perfect".to_string()
                                                                    } else {
                                                                        format!("{:.2}", (p.kills + p.assists) as f64 / p.deaths as f64)
                                                                    };
                                                                    let p_result_cls = if p.win { "text-blue-400" } else { "text-red-400" };
                                                                    let p_result = if p.win { "Win" } else { "Loss" };
                                                                    view! {
                                                                        <tr class="border-b border-divider/20 last:border-0">
                                                                            <td class="py-2 text-primary font-medium">{p.username}</td>
                                                                            <td class="py-2">
                                                                                <div class="flex items-center gap-2">
                                                                                    <img src=icon_url alt=p.champion.clone() class="w-6 h-6 rounded" />
                                                                                    <span class="text-primary">{p.champion}</span>
                                                                                </div>
                                                                            </td>
                                                                            <td class="py-2 text-center">
                                                                                <div class="text-primary">{kda_str}</div>
                                                                                <div class="text-dimmed text-xs">{format!("{kda_ratio} KDA")}</div>
                                                                            </td>
                                                                            <td class="py-2 text-center text-secondary">{p.cs}</td>
                                                                            <td class="py-2 text-center text-secondary">{p.vision_score}</td>
                                                                            <td class="py-2 text-center text-secondary">{format_damage(p.damage)}</td>
                                                                            <td class=format!("py-2 text-center font-medium {p_result_cls}")>{p_result}</td>
                                                                        </tr>
                                                                    }
                                                                }).collect_view()}
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <div></div> }.into_any()
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
