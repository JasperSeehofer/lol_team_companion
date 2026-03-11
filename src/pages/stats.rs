use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::components::stat_card::StatCard;
use crate::components::ui::ErrorBanner;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TeamMatchRow {
    pub match_db_id: String,
    pub user_id: String,
    pub username: String,
    pub riot_match_id: String,
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
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let rows = db::get_team_match_stats(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Convert db type to page type
    Ok(rows.into_iter().map(|r| TeamMatchRow {
        match_db_id: r.match_db_id,
        user_id: r.user_id,
        username: r.username,
        riot_match_id: r.riot_match_id,
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
    }).collect())
}

#[server]
pub async fn sync_team_stats() -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use crate::server::riot;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if !riot::has_api_key() {
        return Err(ServerFnError::new("RIOT_API_KEY not configured. Set it in .env to enable match syncing."));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    let members = db::get_roster_puuids(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut total_synced = 0usize;
    let mut synced_players = 0usize;
    let mut errors = Vec::new();

    for (uid, username, puuid_opt) in &members {
        let puuid = match puuid_opt {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };

        match riot::fetch_match_history(puuid, 440).await {
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

    let linked_count = members.iter().filter(|(_, _, p)| p.as_ref().map(|s| !s.is_empty()).unwrap_or(false)).count();
    let mut msg = format!("Synced {total_synced} matches for {synced_players}/{linked_count} linked players.");
    if !errors.is_empty() {
        msg.push_str(&format!(" Errors: {}", errors.join("; ")));
    }
    Ok(msg)
}

/// A grouped view of a single match with all participating roster members
#[derive(Clone)]
struct MatchGroup {
    #[allow(dead_code)]
    riot_match_id: String,
    game_end: Option<String>,
    game_duration: i32,
    win: bool,
    players: Vec<TeamMatchRow>,
}

fn group_matches(rows: &[TeamMatchRow]) -> Vec<MatchGroup> {
    let mut by_match: HashMap<String, Vec<TeamMatchRow>> = HashMap::new();
    for r in rows {
        by_match.entry(r.riot_match_id.clone()).or_default().push(r.clone());
    }

    let mut groups: Vec<MatchGroup> = by_match.into_iter().map(|(riot_id, players)| {
        let first = &players[0];
        MatchGroup {
            riot_match_id: riot_id,
            game_end: first.game_end.clone(),
            game_duration: first.game_duration,
            win: first.win,
            players,
        }
    }).collect();

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
            if d.len() >= 10 { d[..10].to_string() } else { d.clone() }
        }
        None => "Unknown".to_string(),
    }
}

#[component]
pub fn StatsPage() -> impl IntoView {
    let api_key = Resource::new(|| (), |_| check_api_key());
    let stats = Resource::new(|| (), |_| get_team_stats());
    let (sync_result, set_sync_result) = signal(Option::<Result<String, String>>::None);
    let (syncing, set_syncing) = signal(false);

    // Filters
    let (min_players, set_min_players) = signal(2_usize); // minimum team members in a match
    let (filter_player, set_filter_player) = signal(String::new()); // empty = all players

    let do_sync = move |_| {
        set_syncing.set(true);
        set_sync_result.set(None);
        leptos::task::spawn_local(async move {
            match sync_team_stats().await {
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
                <button
                    class=move || if syncing.get() {
                        "bg-overlay-strong text-muted font-semibold rounded-lg px-5 py-2.5 text-sm cursor-not-allowed"
                    } else {
                        "bg-blue-500 hover:bg-blue-400 text-white font-semibold rounded-lg px-5 py-2.5 text-sm transition-colors"
                    }
                    on:click=do_sync
                    disabled=move || syncing.get()
                >
                    {move || if syncing.get() { "Syncing..." } else { "Sync Matches" }}
                </button>
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

            // Sync result
            {move || sync_result.get().map(|r| match r {
                Ok(msg) => view! {
                    <div class="bg-emerald-500/10 border border-emerald-500/30 rounded-xl p-3 text-emerald-400 text-sm">{msg}</div>
                }.into_any(),
                Err(msg) => view! {
                    <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-3 text-red-400 text-sm">{msg}</div>
                }.into_any(),
            })}

            // Stats content
            <Suspense fallback=|| view! { <div class="text-dimmed text-center py-8">"Loading stats..."</div> }>
                {move || stats.get().map(|result| match result {
                    Err(e) => view! {
                        <ErrorBanner message=format!("Failed to load stats: {e}") />
                    }.into_any(),
                    Ok(rows) if rows.is_empty() => view! {
                        <div class="flex flex-col items-center justify-center py-16">
                            <p class="text-muted text-lg mb-2">"No match data yet"</p>
                            <p class="text-dimmed text-sm">"Click Sync Matches to pull recent games from the Riot API."</p>
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
) -> impl IntoView {
    let all_matches = StoredValue::new(all_matches);
    let unique_players_for_filter = unique_players.clone();

    let filtered = move || {
        let min = min_players.get();
        let player_filter = filter_player.get();

        all_matches.with_value(|matches| {
            matches.iter().filter(|m| {
                // Minimum players filter
                if m.players.len() < min {
                    return false;
                }
                // Player filter
                if !player_filter.is_empty() {
                    if !m.players.iter().any(|p| p.username == player_filter) {
                        return false;
                    }
                }
                true
            }).cloned().collect::<Vec<_>>()
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

        // Aggregate KDA across all player entries in filtered matches
        let all_players: Vec<&TeamMatchRow> = matches.iter().flat_map(|m| m.players.iter()).collect();
        let total_entries = all_players.len();
        let avg_kda = if total_entries == 0 {
            "N/A".to_string()
        } else {
            let k: f64 = all_players.iter().map(|p| p.kills as f64).sum::<f64>() / total_entries as f64;
            let d: f64 = all_players.iter().map(|p| p.deaths as f64).sum::<f64>() / total_entries as f64;
            let a: f64 = all_players.iter().map(|p| p.assists as f64).sum::<f64>() / total_entries as f64;
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

                // Min players dropdown
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

            // Match list
            <div>
                <h2 class="text-primary font-semibold text-lg mb-3">"Match History"</h2>
                <div class="flex flex-col gap-2">
                    {move || {
                        let matches = filtered();
                        if matches.is_empty() {
                            return view! {
                                <p class="text-dimmed text-sm py-4 text-center">"No matches match current filters."</p>
                            }.into_any();
                        }
                        view! {
                            <div class="flex flex-col gap-2">
                                {matches.into_iter().map(|m| {
                                    let win = m.win;
                                    let border_cls = if win { "border-l-emerald-500" } else { "border-l-red-500" };
                                    let result_text = if win { "Victory" } else { "Defeat" };
                                    let result_cls = if win { "text-emerald-400" } else { "text-red-400" };
                                    let date = format_date(&m.game_end);
                                    let duration = format_duration(m.game_duration);
                                    let player_count = m.players.len();

                                    view! {
                                        <div class=format!("bg-elevated/50 border border-divider/50 border-l-4 {border_cls} rounded-xl p-4")>
                                            // Match header
                                            <div class="flex items-center justify-between mb-3">
                                                <div class="flex items-center gap-3">
                                                    <span class=format!("font-semibold text-sm {result_cls}")>{result_text}</span>
                                                    <span class="text-dimmed text-xs">{duration}</span>
                                                    <span class="text-dimmed text-xs">{date}</span>
                                                </div>
                                                <span class={
                                                    if player_count >= 5 {
                                                        "text-emerald-400/70 text-xs font-medium bg-emerald-400/10 px-2 py-0.5 rounded-full"
                                                    } else {
                                                        "text-dimmed text-xs"
                                                    }
                                                }>
                                                    {format!("{player_count} players")}
                                                </span>
                                            </div>
                                            // Players grid
                                            <div class="grid grid-cols-5 gap-2">
                                                {m.players.into_iter().map(|p| {
                                                    let kda_str = format!("{}/{}/{}", p.kills, p.deaths, p.assists);
                                                    let kda_ratio = if p.deaths == 0 {
                                                        "Perfect".to_string()
                                                    } else {
                                                        format!("{:.1}", (p.kills + p.assists) as f64 / p.deaths as f64)
                                                    };
                                                    view! {
                                                        <div class="bg-surface/50 rounded-lg p-2.5">
                                                            <div class="text-muted text-xs truncate">{p.username}</div>
                                                            <div class="text-primary text-sm font-medium truncate mt-0.5">{p.champion}</div>
                                                            <div class="text-secondary text-xs mt-1">{kda_str}</div>
                                                            <div class="flex items-center gap-2 mt-1">
                                                                <span class="text-dimmed text-xs">{format!("{} CS", p.cs)}</span>
                                                                <span class="text-overlay-strong text-xs">"|"</span>
                                                                <span class="text-dimmed text-xs">{format!("{kda_ratio} KDA")}</span>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
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
