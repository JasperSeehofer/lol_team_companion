use leptos::prelude::*;
use crate::models::match_data::{ComparisonMode, EventCategory, MatchParticipant, PerformanceStats, TimelineEvent};

#[server]
pub async fn fetch_match_detail(
    match_id: String,
) -> Result<crate::models::match_data::MatchDetail, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::{db, riot};
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if match_id.is_empty() {
        return Err(ServerFnError::new("No match ID provided"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth.user.ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    // Cache hit path
    if let Some((participants, timeline_events, game_duration, game_mode)) =
        db::get_cached_match_detail(&surreal, &match_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let user_pid = participants
            .iter()
            .find(|p| p.puuid == user.riot_puuid.as_deref().unwrap_or(""))
            .map(|p| p.participant_id)
            .unwrap_or(1);
        let performance = riot::compute_performance(&participants, user_pid, game_duration);

        return Ok(crate::models::match_data::MatchDetail {
            match_id: match_id.clone(),
            game_duration,
            game_mode,
            participants,
            timeline_events,
            user_participant_id: user_pid,
            user_puuid: user.riot_puuid.clone().unwrap_or_default(),
            performance,
        });
    }

    // Cache miss: fetch from Riot API
    if !riot::has_api_key() {
        return Err(ServerFnError::new(
            "Riot API key not configured. Ask an admin to add a RIOT_API_KEY.",
        ));
    }

    let platform = riot::platform_route_from_str(user.riot_region.as_deref().unwrap_or("EUW"));
    let full = riot::fetch_full_match_detail(&match_id, platform)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_pid = full
        .participants
        .iter()
        .find(|p| p.puuid == user.riot_puuid.as_deref().unwrap_or(""))
        .map(|p| p.participant_id)
        .unwrap_or(1);
    let performance = riot::compute_performance(&full.participants, user_pid, full.game_duration);

    // Store in cache (non-fatal on failure)
    if let Err(e) = db::store_match_detail(
        &surreal,
        &match_id,
        &full.participants,
        &full.raw_timeline_events,
        full.game_duration,
        &full.game_mode,
    )
    .await
    {
        tracing::warn!("Failed to cache match detail for {match_id}: {e}");
    }

    Ok(crate::models::match_data::MatchDetail {
        match_id,
        game_duration: full.game_duration,
        game_mode: full.game_mode,
        participants: full.participants,
        timeline_events: full.raw_timeline_events,
        user_participant_id: user_pid,
        user_puuid: user.riot_puuid.unwrap_or_default(),
        performance,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn format_gold(gold: i32) -> String {
    if gold >= 1000 {
        format!("{:.1}k", gold as f64 / 1000.0)
    } else {
        gold.to_string()
    }
}

fn format_duration(secs: i32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m}:{s:02}")
}

fn timeline_pct(event_timestamp_ms: i64, game_duration_secs: i32) -> f64 {
    let game_ms = game_duration_secs as f64 * 1000.0;
    if game_ms <= 0.0 {
        return 0.0;
    }
    ((event_timestamp_ms as f64 / game_ms) * 100.0).clamp(0.0, 100.0)
}

fn event_tooltip(event: &TimelineEvent, participants: &[MatchParticipant]) -> String {
    let time_min = event.timestamp_ms / 60_000;
    let time_sec = (event.timestamp_ms % 60_000) / 1000;
    let time_str = format!("{time_min}:{time_sec:02}");
    match event.event_type.as_str() {
        "ELITE_MONSTER_KILL" => {
            let monster = event.monster_type.as_deref().unwrap_or("Monster");
            let sub = event
                .monster_sub_type
                .as_deref()
                .map(|s| format!(" ({s})"))
                .unwrap_or_default();
            format!("{time_str} - {monster}{sub} killed")
        }
        "BUILDING_KILL" => {
            let building = event.building_type.as_deref().unwrap_or("Building");
            format!("{time_str} - {building} destroyed")
        }
        "CHAMPION_KILL" => {
            let killer = event
                .killer_participant_id
                .and_then(|id| participants.iter().find(|p| p.participant_id == id))
                .map(|p| p.summoner_name.as_str())
                .unwrap_or("Unknown");
            let victim = event
                .victim_participant_id
                .and_then(|id| participants.iter().find(|p| p.participant_id == id))
                .map(|p| p.summoner_name.as_str())
                .unwrap_or("Unknown");
            let mut text = format!("{time_str} - {killer} killed {victim}");
            if event.is_first_blood {
                text.push_str(" (First Blood)");
            }
            if let Some(mk) = event.multi_kill_length {
                if mk >= 2 {
                    text.push_str(&format!(" ({mk}x kill)"));
                }
            }
            text
        }
        "WARD_PLACED" => format!("{time_str} - Ward placed"),
        "TEAMFIGHT" => format!(
            "{time_str} - Teamfight ({} players)",
            event.involved_participants.len()
        ),
        _ => format!("{time_str} - {}", event.event_type),
    }
}

// ---------------------------------------------------------------------------
// Scoreboard sub-components
// ---------------------------------------------------------------------------

#[component]
fn ItemIcon(item_id: i32) -> impl IntoView {
    if item_id > 0 {
        view! {
            <img
                src=format!("https://ddragon.leagueoflegends.com/cdn/15.6.1/img/item/{item_id}.png")
                class="w-6 h-6 rounded"
                title=format!("Item {item_id}")
            />
        }.into_any()
    } else {
        view! {
            <div class="w-6 h-6 rounded bg-elevated border border-divider/30" />
        }.into_any()
    }
}

#[component]
fn ParticipantRow(
    p: MatchParticipant,
    is_user: bool,
) -> impl IntoView {
    let row_class = if is_user {
        "h-12 flex items-center gap-3 px-3 border-l-4 border-accent bg-accent/10".to_string()
    } else if p.win {
        "h-12 flex items-center gap-3 px-3 bg-blue-500/5".to_string()
    } else {
        "h-12 flex items-center gap-3 px-3 bg-red-500/5".to_string()
    };

    let champ_icon = champion_icon_url(&p.champion_name);
    let champ_name = p.champion_name.clone();
    let summoner = p.summoner_name.clone();
    let kda = format!("{}/{}/{}", p.kills, p.deaths, p.assists);
    let dmg = format_damage(p.damage);
    let gold = format_gold(p.gold_earned);
    let items = p.items;

    view! {
        <div class=row_class>
            // Champion icon (36px)
            <div class="w-9 shrink-0">
                <img src=champ_icon alt=champ_name.clone() class="w-7 h-7 rounded-full" />
            </div>
            // Summoner name (flex-1, min 160px)
            <div class="flex-1 min-w-[160px] truncate">
                <span class="text-sm text-primary">{summoner}</span>
            </div>
            // KDA (80px)
            <div class="w-20 text-center shrink-0">
                <span class="text-sm font-semibold text-primary">{kda}</span>
            </div>
            // Items (180px)
            <div class="w-[180px] flex items-center gap-1 shrink-0">
                <ItemIcon item_id=items[0] />
                <ItemIcon item_id=items[1] />
                <ItemIcon item_id=items[2] />
                <ItemIcon item_id=items[3] />
                <ItemIcon item_id=items[4] />
                <ItemIcon item_id=items[5] />
            </div>
            // Damage (80px)
            <div class="w-20 text-center shrink-0">
                <span class="text-sm text-secondary">{dmg}</span>
            </div>
            // Gold (80px)
            <div class="w-20 text-center shrink-0">
                <span class="text-sm text-secondary">{gold}</span>
            </div>
            // Vision (64px)
            <div class="w-16 text-center shrink-0">
                <span class="text-sm text-secondary">{p.vision_score}</span>
            </div>
        </div>
    }
}

#[component]
fn TeamScoreboard(
    title: String,
    team_win: bool,
    participants: Vec<MatchParticipant>,
    user_participant_id: i32,
    team_color: &'static str, // "blue" or "red"
) -> impl IntoView {
    let header_cls = if team_color == "blue" {
        "bg-blue-500/20 border border-blue-500/40 rounded-t-xl p-3 flex items-center justify-between"
    } else {
        "bg-red-500/20 border border-red-500/40 rounded-t-xl p-3 flex items-center justify-between"
    };

    let result_cls = if team_color == "blue" {
        "text-xs uppercase tracking-wider text-blue-400"
    } else {
        "text-xs uppercase tracking-wider text-red-400"
    };

    let result_text = if team_win { "Victory" } else { "Defeat" };

    view! {
        <div class="bg-surface border border-divider rounded-xl overflow-hidden">
            // Team header
            <div class=header_cls>
                <h2 class="text-xl font-semibold text-primary">{title}</h2>
                <span class=result_cls>{result_text}</span>
            </div>
            // Column headers
            <div class="flex items-center gap-3 px-3 py-2 border-b border-divider/50">
                <div class="w-9 shrink-0"></div>
                <div class="flex-1 min-w-[160px]">
                    <span class="text-xs font-normal text-muted uppercase tracking-wider">"Summoner"</span>
                </div>
                <div class="w-20 text-center shrink-0">
                    <span class="text-xs font-normal text-muted uppercase tracking-wider">"KDA"</span>
                </div>
                <div class="w-[180px] shrink-0">
                    <span class="text-xs font-normal text-muted uppercase tracking-wider">"Items"</span>
                </div>
                <div class="w-20 text-center shrink-0">
                    <span class="text-xs font-normal text-muted uppercase tracking-wider">"Damage"</span>
                </div>
                <div class="w-20 text-center shrink-0">
                    <span class="text-xs font-normal text-muted uppercase tracking-wider">"Gold"</span>
                </div>
                <div class="w-16 text-center shrink-0">
                    <span class="text-xs font-normal text-muted uppercase tracking-wider">"Vision"</span>
                </div>
            </div>
            // Participant rows
            {participants.into_iter().map(|p| {
                let is_user = p.participant_id == user_participant_id;
                view! { <ParticipantRow p=p is_user=is_user /> }
            }).collect_view()}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Performance bar helpers
// ---------------------------------------------------------------------------

fn performance_verdict(user_val: f32, avg: f32) -> &'static str {
    if avg == 0.0 {
        return "Average";
    }
    if user_val > avg * 1.1 {
        "Above average"
    } else if user_val < avg * 0.9 {
        "Below average"
    } else {
        "Average"
    }
}

fn normalize_pct(user_val: f32, avg: f32) -> f32 {
    let max = (user_val).max(avg * 2.0).max(1.0);
    (user_val / max * 100.0).min(100.0).max(0.0)
}

fn avg_marker_pct(user_val: f32, avg: f32) -> f32 {
    let max = (user_val).max(avg * 2.0).max(1.0);
    (avg / max * 100.0).min(100.0).max(0.0)
}

#[component]
fn PerformanceBar(
    label: &'static str,
    user_val: f32,
    avg: f32,
    display_val: String,
) -> impl IntoView {
    let fill_pct = normalize_pct(user_val, avg);
    let marker_pct = avg_marker_pct(user_val, avg);
    let verdict = performance_verdict(user_val, avg);

    view! {
        <div class="flex items-center gap-3 mb-3">
            // Label (120px)
            <span class="text-sm text-secondary w-[120px] shrink-0">{label}</span>
            // Bar track (flex-1)
            <div class="bg-elevated h-4 rounded-full relative flex-1">
                // Bar fill
                <div
                    class="bg-accent/70 h-full rounded-full"
                    style=format!("width: {}%", fill_pct)
                />
                // Average marker
                <div
                    class="absolute top-0 h-full w-0.5 bg-muted/50"
                    style=format!("left: {}%", marker_pct)
                />
            </div>
            // Number (64px)
            <span class="text-sm font-semibold text-primary w-16 text-right shrink-0">{display_val}</span>
            // Verdict (80px)
            <span class="text-xs text-muted w-20 shrink-0">{verdict}</span>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Performance section
// ---------------------------------------------------------------------------

#[component]
fn PerformanceSection(
    perf: PerformanceStats,
    _game_duration_secs: i32,
    comparison_mode: ReadSignal<ComparisonMode>,
    set_comparison_mode: WriteSignal<ComparisonMode>,
    user_champion: String,
    opponent_champion: String,
) -> impl IntoView {
    let has_lane_opponent = perf.lane_opponent_damage.is_some();

    let perf_stored = StoredValue::new(perf);

    view! {
        <div class="bg-surface border border-divider rounded-xl p-4">
            <h2 class="text-xl font-semibold text-primary mb-3">"My Performance"</h2>

            // Comparison toggle
            <div class="flex gap-2 mb-4">
                <button
                    class=move || if comparison_mode.get() == ComparisonMode::GameAverage {
                        "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                    } else {
                        "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                    }
                    on:click=move |_| set_comparison_mode.set(ComparisonMode::GameAverage)
                >
                    "vs Game Average"
                </button>
                <button
                    class=move || {
                        let base = if comparison_mode.get() == ComparisonMode::LaneOpponent {
                            "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                        } else {
                            "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                        };
                        if !has_lane_opponent {
                            format!("{base} opacity-40 cursor-not-allowed")
                        } else {
                            base.to_string()
                        }
                    }
                    on:click=move |_| {
                        if has_lane_opponent {
                            set_comparison_mode.set(ComparisonMode::LaneOpponent);
                        }
                    }
                    title=if has_lane_opponent { "" } else { "Lane role data not available for this match" }
                >
                    "vs Lane Opponent"
                </button>
            </div>

            // Performance bars
            {move || {
                perf_stored.with_value(|perf| {
                    let use_opponent = comparison_mode.get() == ComparisonMode::LaneOpponent && has_lane_opponent;

                    // Damage share bar (out of 100%)
                    let dmg_avg: f32 = 10.0; // 100%/10 players
                    let dmg_display = format!("{:.1}%", perf.damage_share_pct);

                    // Vision score bar
                    let vision_avg = if use_opponent {
                        perf.lane_opponent_vision.unwrap_or(0) as f32
                    } else {
                        perf.vision_score_avg
                    };
                    let vision_display = perf.vision_score.to_string();

                    // CS per minute bar
                    let cs_avg = if use_opponent {
                        perf.lane_opponent_cs_per_min.unwrap_or(0.0)
                    } else {
                        perf.cs_per_min_avg
                    };
                    let cs_display = format!("{:.1}", perf.cs_per_min);

                    // Gold bar
                    let gold_avg = if use_opponent {
                        perf.lane_opponent_gold.unwrap_or(0) as f32
                    } else {
                        perf.gold_earned_avg
                    };
                    let gold_display = format_gold(perf.gold_earned);

                    view! {
                        <div>
                            <PerformanceBar
                                label="Damage Share"
                                user_val=perf.damage_share_pct
                                avg=dmg_avg
                                display_val=dmg_display
                            />
                            <PerformanceBar
                                label="Vision Score"
                                user_val=perf.vision_score as f32
                                avg=vision_avg
                                display_val=vision_display
                            />
                            <PerformanceBar
                                label="CS per Minute"
                                user_val=perf.cs_per_min
                                avg=cs_avg
                                display_val=cs_display
                            />
                            <PerformanceBar
                                label="Gold Earned"
                                user_val=perf.gold_earned as f32
                                avg=gold_avg
                                display_val=gold_display
                            />
                        </div>
                    }
                })
            }}

            // Add Learning CTA
            <a
                href=format!("/personal-learnings/new?champion={}&opponent={}", user_champion, opponent_champion)
                class="mt-4 inline-flex items-center gap-2 bg-accent hover:bg-accent-hover text-accent-contrast px-4 py-2 rounded-lg text-sm font-semibold transition-colors"
            >
                "Add Learning ->"
            </a>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Main page component
// ---------------------------------------------------------------------------

#[component]
pub fn MatchDetailPage() -> impl IntoView {
    use crate::components::ui::{ErrorBanner, SkeletonCard};
    use leptos_router::hooks::use_params_map;

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

    // Route param extraction
    let params = use_params_map();
    let match_id = move || params.read().get("id").unwrap_or_default();
    let detail = Resource::new(match_id, |id| fetch_match_detail(id));

    // Signals for interactivity
    let (comparison_mode, set_comparison_mode) = signal(ComparisonMode::GameAverage);
    let (retry_count, set_retry_count) = signal(0u32);

    // Timeline filter signals
    let (show_objectives, set_show_objectives) = signal(true);
    let (show_towers, set_show_towers) = signal(true);
    let (show_kills, set_show_kills) = signal(true);
    let (show_wards, set_show_wards) = signal(true);
    let (show_teamfights, set_show_teamfights) = signal(true);
    let (show_recalls, set_show_recalls) = signal(true);
    let (selected_event, set_selected_event) = signal(Option::<usize>::None);

    view! {
        <div class="max-w-6xl mx-auto px-6 py-6">
            // Back link
            <a href="/stats" class="text-muted hover:text-secondary text-sm">"<- Back to history"</a>

            <div class="mt-4">
                <Suspense fallback=move || view! {
                    <div class="flex flex-col gap-6">
                        <SkeletonCard height="h-12" />
                        <SkeletonCard height="h-48" />
                        <SkeletonCard height="h-48" />
                        <SkeletonCard height="h-16" />
                        <SkeletonCard height="h-32" />
                    </div>
                }>
                    {move || {
                        let _ = retry_count.get(); // track retry
                        detail.get().map(|result| match result {
                            Err(e) => {
                                let err_msg = format!("Failed to load match: {e}");
                                view! {
                                    <div class="flex flex-col gap-4">
                                        <ErrorBanner message=err_msg />
                                        <button
                                            class="self-start bg-elevated border border-outline text-secondary hover:text-primary px-4 py-2 rounded-lg text-sm transition-colors"
                                            on:click=move |_| {
                                                detail.refetch();
                                                set_retry_count.update(|c| *c += 1);
                                            }
                                        >
                                            "Retry"
                                        </button>
                                    </div>
                                }.into_any()
                            }
                            Ok(d) => {
                                let d_stored = StoredValue::new(d.clone());

                                // Find user's champion and opponent
                                let user_participant = d.participants.iter()
                                    .find(|p| p.participant_id == d.user_participant_id)
                                    .cloned();
                                let user_champion = user_participant.as_ref()
                                    .map(|p| p.champion_name.clone())
                                    .unwrap_or_default();
                                let user_position = user_participant.as_ref()
                                    .map(|p| p.team_position.clone())
                                    .unwrap_or_default();
                                let user_team_id = user_participant.as_ref()
                                    .map(|p| p.team_id)
                                    .unwrap_or(100);

                                // Find lane opponent (same position, opposing team)
                                let opponent_team_id = if user_team_id == 100 { 200 } else { 100 };
                                let opponent_champion = if user_position.is_empty() {
                                    String::new()
                                } else {
                                    d.participants.iter()
                                        .find(|p| p.team_id == opponent_team_id && p.team_position == user_position)
                                        .map(|p| p.champion_name.clone())
                                        .unwrap_or_default()
                                };

                                // Split teams
                                let blue_team: Vec<MatchParticipant> = d.participants.iter()
                                    .filter(|p| p.team_id == 100)
                                    .cloned()
                                    .collect();
                                let red_team: Vec<MatchParticipant> = d.participants.iter()
                                    .filter(|p| p.team_id == 200)
                                    .cloned()
                                    .collect();

                                let blue_win = blue_team.first().map(|p| p.win).unwrap_or(false);
                                let red_win = red_team.first().map(|p| p.win).unwrap_or(false);

                                // Page header info
                                let user_kda = user_participant.as_ref()
                                    .map(|p| format!("{}/{}/{}", p.kills, p.deaths, p.assists))
                                    .unwrap_or_default();
                                let user_win = user_participant.as_ref()
                                    .map(|p| p.win)
                                    .unwrap_or(false);

                                let win_badge_cls = if user_win {
                                    "bg-blue-500/20 text-blue-400 text-sm font-bold px-3 py-1 rounded"
                                } else {
                                    "bg-red-500/20 text-red-400 text-sm font-bold px-3 py-1 rounded"
                                };
                                let win_text = if user_win { "Victory" } else { "Defeat" };

                                let perf = d_stored.with_value(|d| d.performance.clone());
                                let game_duration = d.game_duration;
                                let user_pid = d.user_participant_id;

                                view! {
                                    <div class="flex flex-col gap-6">
                                        // Page header
                                        <div class="flex items-center gap-4">
                                            <h1 class="text-3xl font-semibold text-primary">"Match Detail"</h1>
                                            {user_participant.map(|p| {
                                                let icon = champion_icon_url(&p.champion_name);
                                                let champ = p.champion_name.clone();
                                                view! {
                                                    <img src=icon alt=champ.clone() class="w-7 h-7 rounded-full" />
                                                    <span class="text-primary text-sm">{champ}</span>
                                                }
                                            })}
                                            <span class="text-secondary text-sm">{user_kda}</span>
                                            <span class=win_badge_cls>{win_text}</span>
                                            <span class="text-muted text-xs ml-2">
                                                {format!("{} · {}", d.game_mode, format_duration(d.game_duration))}
                                            </span>
                                        </div>

                                        // Blue team scoreboard
                                        <TeamScoreboard
                                            title="Blue Team".to_string()
                                            team_win=blue_win
                                            participants=blue_team
                                            user_participant_id=user_pid
                                            team_color="blue"
                                        />

                                        // Red team scoreboard
                                        <TeamScoreboard
                                            title="Red Team".to_string()
                                            team_win=red_win
                                            participants=red_team
                                            user_participant_id=user_pid
                                            team_color="red"
                                        />

                                        // Timeline section
                                        {
                                            let timeline_events = d_stored.with_value(|d| d.timeline_events.clone());
                                            let participants_for_timeline = d_stored.with_value(|d| d.participants.clone());
                                            let participants_for_detail = participants_for_timeline.clone();

                                            view! {
                                                <div class="bg-surface border border-divider rounded-xl p-4">
                                                    <h2 class="text-xl font-semibold text-primary mb-3">"Timeline"</h2>

                                                    // Filter toggles row
                                                    <div class="flex gap-2 flex-wrap mb-3">
                                                        <button
                                                            class=move || if show_objectives.get() {
                                                                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                                                            } else {
                                                                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                                                            }
                                                            on:click=move |_| set_show_objectives.set(!show_objectives.get_untracked())
                                                        >"Objectives"</button>
                                                        <button
                                                            class=move || if show_towers.get() {
                                                                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                                                            } else {
                                                                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                                                            }
                                                            on:click=move |_| set_show_towers.set(!show_towers.get_untracked())
                                                        >"Towers"</button>
                                                        <button
                                                            class=move || if show_kills.get() {
                                                                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                                                            } else {
                                                                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                                                            }
                                                            on:click=move |_| set_show_kills.set(!show_kills.get_untracked())
                                                        >"Kills"</button>
                                                        <button
                                                            class=move || if show_wards.get() {
                                                                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                                                            } else {
                                                                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                                                            }
                                                            on:click=move |_| set_show_wards.set(!show_wards.get_untracked())
                                                        >"Wards"</button>
                                                        <button
                                                            class=move || if show_recalls.get() {
                                                                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                                                            } else {
                                                                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                                                            }
                                                            on:click=move |_| set_show_recalls.set(!show_recalls.get_untracked())
                                                        >"Recalls"</button>
                                                        <button
                                                            class=move || if show_teamfights.get() {
                                                                "bg-accent text-accent-contrast text-xs px-3 py-1.5 rounded-full font-semibold"
                                                            } else {
                                                                "bg-elevated border border-divider text-muted text-xs px-3 py-1.5 rounded-full hover:border-outline hover:text-secondary transition-colors"
                                                            }
                                                            on:click=move |_| set_show_teamfights.set(!show_teamfights.get_untracked())
                                                        >"Teamfights"</button>
                                                    </div>

                                                    // Timeline bar track
                                                    <div class="relative w-full h-10 bg-elevated border border-divider rounded-lg">
                                                        {move || {
                                                            let events = timeline_events.clone();
                                                            let participants_ref = participants_for_timeline.clone();
                                                            let visible: Vec<(usize, crate::models::match_data::TimelineEvent)> = events
                                                                .iter()
                                                                .enumerate()
                                                                .filter(|(_, e)| match e.category {
                                                                    EventCategory::Objective => show_objectives.get(),
                                                                    EventCategory::Tower => show_towers.get(),
                                                                    EventCategory::Kill => show_kills.get(),
                                                                    EventCategory::Ward => show_wards.get(),
                                                                    EventCategory::Teamfight => show_teamfights.get(),
                                                                    EventCategory::Recall => show_recalls.get(),
                                                                })
                                                                .map(|(idx, e)| (idx, e.clone()))
                                                                .collect();

                                                            if visible.is_empty() {
                                                                return view! {
                                                                    <p class="text-sm text-muted text-center py-4 absolute inset-0 flex items-center justify-center">
                                                                        "No events match the current filters."
                                                                    </p>
                                                                }.into_any();
                                                            }

                                                            visible.into_iter().map(|(idx, event)| {
                                                                let left_pct = timeline_pct(event.timestamp_ms, game_duration);
                                                                let tooltip = event_tooltip(&event, &participants_ref);

                                                                let (size_class, shape_class) = match event.category {
                                                                    EventCategory::Objective => {
                                                                        match event.monster_type.as_deref() {
                                                                            Some(m) if m.contains("BARON") || m.contains("HORDE") =>
                                                                                ("w-4 h-4", "rounded-full border-2"),
                                                                            _ => ("w-3 h-3", "rounded-full"),
                                                                        }
                                                                    }
                                                                    EventCategory::Tower => ("w-2 h-2", "rounded-sm"),
                                                                    EventCategory::Kill => ("w-2 h-2", "rounded-full"),
                                                                    EventCategory::Ward => ("w-2 h-2", "rounded-full"),
                                                                    EventCategory::Recall => ("w-2 h-2", "rounded-full"),
                                                                    EventCategory::Teamfight => ("w-4 h-4", "rounded-full"),
                                                                };

                                                                let team_color = match event.team_id {
                                                                    Some(100) => "bg-blue-400 border-blue-500",
                                                                    Some(200) => "bg-red-400 border-red-500",
                                                                    _ => "bg-muted border-muted",
                                                                };

                                                                let is_user_event = event.killer_participant_id == Some(user_pid)
                                                                    || event.involved_participants.contains(&user_pid);
                                                                let user_ring = if is_user_event {
                                                                    " ring-2 ring-accent ring-offset-1 ring-offset-base"
                                                                } else {
                                                                    ""
                                                                };

                                                                let btn_class = format!(
                                                                    "absolute top-1/2 -translate-y-1/2 -translate-x-1/2 {size_class} {shape_class} {team_color}{user_ring} cursor-pointer transition-transform z-10"
                                                                );
                                                                let btn_class_selected = format!(
                                                                    "absolute top-1/2 -translate-y-1/2 -translate-x-1/2 {size_class} {shape_class} {team_color}{user_ring} cursor-pointer transition-transform scale-150 z-20"
                                                                );

                                                                view! {
                                                                    <button
                                                                        class=move || if selected_event.get() == Some(idx) {
                                                                            btn_class_selected.clone()
                                                                        } else {
                                                                            btn_class.clone()
                                                                        }
                                                                        style=format!("left: {left_pct:.2}%")
                                                                        title=tooltip
                                                                        on:click=move |ev| {
                                                                            ev.stop_propagation();
                                                                            let current = selected_event.get_untracked();
                                                                            if current == Some(idx) {
                                                                                set_selected_event.set(None);
                                                                            } else {
                                                                                set_selected_event.set(Some(idx));
                                                                            }
                                                                        }
                                                                    />
                                                                }
                                                            }).collect_view().into_any()
                                                        }}
                                                    </div>

                                                    // Event detail panel
                                                    {move || {
                                                        let events = d_stored.with_value(|d| d.timeline_events.clone());
                                                        let participants_ref = participants_for_detail.clone();
                                                        if let Some(idx) = selected_event.get() {
                                                            if let Some(event) = events.get(idx) {
                                                                let detail_text = event_tooltip(event, &participants_ref);
                                                                let involved_names: Vec<String> = event.involved_participants.iter()
                                                                    .filter_map(|id| participants_ref.iter().find(|p| p.participant_id == *id))
                                                                    .map(|p| p.summoner_name.clone())
                                                                    .collect();
                                                                let has_involved = !involved_names.is_empty();
                                                                let names_str = involved_names.join(", ");
                                                                return view! {
                                                                    <div class="bg-surface border border-divider rounded-lg p-4 mt-3">
                                                                        <p class="text-sm text-secondary">{detail_text}</p>
                                                                        {if has_involved {
                                                                            view! {
                                                                                <p class="text-xs text-muted mt-2">
                                                                                    "Involved: " {names_str}
                                                                                </p>
                                                                            }.into_any()
                                                                        } else {
                                                                            view! { <span /> }.into_any()
                                                                        }}
                                                                    </div>
                                                                }.into_any();
                                                            }
                                                        }
                                                        view! { <span /> }.into_any()
                                                    }}
                                                </div>
                                            }
                                        }

                                        // Performance section
                                        <PerformanceSection
                                            perf=perf
                                            _game_duration_secs=game_duration
                                            comparison_mode=comparison_mode
                                            set_comparison_mode=set_comparison_mode
                                            user_champion=user_champion
                                            opponent_champion=opponent_champion
                                        />
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
