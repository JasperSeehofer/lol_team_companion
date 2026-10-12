//! Match detail: one game's scoreboard per side, the timeline with event
//! markers shaped by category, and the signed-in player's performance
//! against the game average or their lane opponent.

use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, Callout, CalloutTone, ChampionAvatar, DraftSide,
    GameResult, GlyphSize, PageHeader, PageShell, ResultGlyph, Role, RoleLabel, SectionHeader,
    Segment, SegmentSize, SegmentedControl, SideMarker, SideText, Skeleton, SkeletonGroup,
    SkeletonShape, Table, Td, Th, Tr,
};
use crate::models::match_data::{
    EventCategory, MatchDetail, MatchParticipant, PerformanceStats, TimelineEvent,
};
use leptos::prelude::*;

#[server]
pub async fn fetch_match_detail(
    match_id: String,
) -> Result<crate::models::match_data::MatchDetail, ServerFnError> {
    use crate::server::{authz, db, rate_limit::Bucket, riot};

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    if match_id.is_empty() {
        return Err(ServerFnError::new("No match ID provided"));
    }
    // Riot match IDs look like "EUW1_7123456789".
    let valid_id = match_id.split_once('_').is_some_and(|(platform, num)| {
        !platform.is_empty()
            && platform.chars().all(|c| c.is_ascii_alphanumeric())
            && !num.is_empty()
            && num.chars().all(|c| c.is_ascii_digit())
    });
    if !valid_id || match_id.len() > 32 {
        return Err(ServerFnError::new(authz::NOT_FOUND));
    }

    // Cache hit path
    if let Some((participants, timeline_events, game_duration, game_mode)) =
        db::get_cached_match_detail(&surreal, &match_id)
            .await
            .map_err(authz::server_err)?
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
    authz::rate_limit(Bucket::Riot).await?;
    let riot_client = authz::riot()?;
    if !riot_client.is_configured() {
        return Err(ServerFnError::new(
            "Riot API key not configured. Ask an admin to add a RIOT_API_KEY.",
        ));
    }
    // Match + timeline.
    riot_client.ensure_headroom(2).map_err(authz::server_err)?;

    // The match ID names its platform ("EUW1_..."); fall back to the profile.
    let platform = riot::platform_from_match_id(&match_id)
        .unwrap_or_else(|| riot::platform_for(&[user.riot_region.as_deref()]));
    let full = riot_client
        .fetch_full_match_detail(&match_id, platform)
        .await
        .map_err(authz::server_err)?;

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
// Scoreboard
// ---------------------------------------------------------------------------

fn position_role(pos: &str) -> Option<Role> {
    match pos {
        "TOP" => Some(Role::Top),
        "JUNGLE" => Some(Role::Jungle),
        "MIDDLE" => Some(Role::Mid),
        "BOTTOM" => Some(Role::Bot),
        "UTILITY" => Some(Role::Support),
        _ => None,
    }
}

#[component]
fn ItemIcon(item_id: i32) -> impl IntoView {
    if item_id > 0 {
        view! {
            <img
                src=crate::models::ddragon::item_icon_url(item_id)
                class="size-6 rounded-ctl border border-rule"
                alt=format!("Item {item_id}")
                loading="lazy"
                width="24"
                height="24"
            />
        }
        .into_any()
    } else {
        view! { <span class="size-6 rounded-ctl border border-dashed border-rule" aria-hidden="true" /> }.into_any()
    }
}

#[component]
fn TeamScoreboard(
    side: DraftSide,
    team_win: bool,
    participants: Vec<MatchParticipant>,
    user_participant_id: i32,
) -> impl IntoView {
    let id = match side {
        DraftSide::Blue => "blue-side-h",
        DraftSide::Red => "red-side-h",
    };
    view! {
        <section aria-labelledby=id data-scoreboard=side.label().to_lowercase()>
            <h2 id=id class="mb-3 flex items-center gap-3 font-display text-heading font-semibold text-primary">
                <SideMarker side=side text=SideText::Long />
                <ResultGlyph result=GameResult::from_win(team_win) size=GlyphSize::Sm word=true />
            </h2>
            <Table caption=format!("{} side scoreboard", side.label()) caption_hidden=true>
                <thead>
                    <tr>
                        <Th>"Pos."</Th>
                        <Th>"Player"</Th>
                        <Th numeric=true>"K / D / A"</Th>
                        <Th>"Items"</Th>
                        <Th numeric=true>"Damage"</Th>
                        <Th numeric=true>"Gold"</Th>
                        <Th numeric=true>"Vision"</Th>
                    </tr>
                </thead>
                <tbody>
                    {participants.into_iter().map(|p| {
                        let is_user = p.participant_id == user_participant_id;
                        let items = p.items;
                        view! {
                            <Tr selected=is_user>
                                <Td>{match position_role(&p.team_position) { Some(r) => view! { <RoleLabel role=r /> }.into_any(), None => view! { <span class="text-muted">"None"</span> }.into_any() }}</Td>
                                <Td>
                                    <span class="flex min-w-0 items-center gap-2">
                                        <ChampionAvatar name=p.champion_name.clone() size=AvatarSize::Sm decorative=true />
                                        <span class="min-w-0">
                                            <span class="block truncate font-medium text-primary">{p.summoner_name.clone()}{is_user.then_some(" (you)")}</span>
                                            <span class="block truncate text-caption text-muted">{p.champion_name.clone()}</span>
                                        </span>
                                    </span>
                                </Td>
                                <Td numeric=true>{format!("{} / {} / {}", p.kills, p.deaths, p.assists)}</Td>
                                <Td>
                                    <span class="flex gap-1">
                                        {items.into_iter().map(|i| view! { <ItemIcon item_id=i /> }).collect_view()}
                                    </span>
                                </Td>
                                <Td numeric=true>{format_damage(p.damage)}</Td>
                                <Td numeric=true>{format_gold(p.gold_earned)}</Td>
                                <Td numeric=true>{p.vision_score}</Td>
                            </Tr>
                        }
                    }).collect_view()}
                </tbody>
            </Table>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Timeline markers: one shape per category, the side in colour
// ---------------------------------------------------------------------------

/// The marker shape of a timeline category, as SVG in a 16 x 16 box.
fn marker_shape(category: &EventCategory, big: bool) -> AnyView {
    let (fill, stroke) = ("currentColor", "currentColor");
    match category {
        EventCategory::Objective if big => view! { <polygon points="8,0 16,8 8,16 0,8" fill=fill /> }.into_any(),
        EventCategory::Objective => view! { <polygon points="8,2 14,8 8,14 2,8" fill=fill /> }.into_any(),
        EventCategory::Tower => view! { <rect x="3" y="3" width="10" height="10" fill=fill /> }.into_any(),
        EventCategory::Kill => view! { <polygon points="8,2 14,13 2,13" fill=fill /> }.into_any(),
        EventCategory::Ward => view! { <path d="M8 3v10M3 8h10" stroke=stroke stroke-width="2.5" /> }.into_any(),
        EventCategory::Recall => view! { <circle cx="8" cy="8" r="4.5" fill="none" stroke=stroke stroke-width="2" /> }.into_any(),
        EventCategory::Teamfight => view! { <polygon points="4,1 12,1 16,8 12,15 4,15 0,8" fill="none" stroke=stroke stroke-width="2" /> }.into_any(),
    }
}

fn category_word(c: &EventCategory) -> &'static str {
    match c {
        EventCategory::Objective => "Objective",
        EventCategory::Tower => "Tower",
        EventCategory::Kill => "Kill",
        EventCategory::Ward => "Ward",
        EventCategory::Recall => "Recall",
        EventCategory::Teamfight => "Teamfight",
    }
}

const CATEGORIES: [EventCategory; 6] = [
    EventCategory::Objective,
    EventCategory::Tower,
    EventCategory::Kill,
    EventCategory::Ward,
    EventCategory::Recall,
    EventCategory::Teamfight,
];

#[component]
fn Timeline(
    d: MatchDetail,
    user_champion: String,
    opponent_champion: String,
    user_win: bool,
) -> impl IntoView {
    let shown: [RwSignal<bool>; 6] = std::array::from_fn(|_| RwSignal::new(true));
    let selected = RwSignal::new(None::<usize>);
    let duration = d.game_duration;
    let user_pid = d.user_participant_id;
    let events = StoredValue::new(d.timeline_events.clone());
    let participants = StoredValue::new(d.participants.clone());
    let match_id = d.match_id.clone();
    let cat_index = |c: &EventCategory| CATEGORIES.iter().position(|x| x == c).unwrap_or(0);
    view! {
        <section aria-labelledby="timeline-h" data-timeline>
            <SectionHeader title="Timeline" id="timeline-h" meta=format!("0:00 to {}", format_duration(duration)) />
            <div class="mb-3 flex flex-wrap gap-2" role="group" aria-label="Show event kinds">
                {CATEGORIES.iter().enumerate().map(|(i, c)| {
                    let sig = shown[i];
                    view! {
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Secondary pressed=Signal::derive(move || sig.get()) on_click=Callback::new(move |_| sig.update(|v| *v = !*v))>
                            <svg viewBox="0 0 16 16" class="size-3 text-secondary" aria-hidden="true">{marker_shape(c, false)}</svg>
                            {category_word(c)}
                        </Button>
                    }
                }).collect_view()}
            </div>
            <div class="relative h-12 w-full border border-rule bg-ink-1" data-timeline-track>
                <div class="absolute left-3 right-3 top-1/2 h-px -translate-y-1/2 bg-rule" aria-hidden="true"></div>
                {move || {
                    let parts = participants.get_value();
                    let visible: Vec<(usize, TimelineEvent)> = events
                        .get_value()
                        .into_iter()
                        .enumerate()
                        .filter(|(_, e)| shown[cat_index(&e.category)].get())
                        .collect();
                    if visible.is_empty() {
                        return view! { <p class="absolute inset-0 flex items-center justify-center text-label text-muted">"No events of the chosen kinds."</p> }.into_any();
                    }
                    visible.into_iter().map(|(idx, e)| {
                        let left = timeline_pct(e.timestamp_ms, duration);
                        let label = event_tooltip(&e, &parts);
                        let big = matches!(e.monster_type.as_deref(), Some(m) if m.contains("BARON") || m.contains("HORDE"))
                            || e.category == EventCategory::Teamfight;
                        let side = match e.team_id {
                            Some(100) => "text-side-blue",
                            Some(200) => "text-side-red",
                            _ => "text-muted",
                        };
                        let mine = e.killer_participant_id == Some(user_pid) || e.involved_participants.contains(&user_pid);
                        let size = if big { "size-4" } else { "size-3" };
                        let class = format!("absolute top-1/2 grid -translate-x-1/2 -translate-y-1/2 cursor-pointer place-items-center rounded-ctl p-0.5 {side}");
                        let shape = marker_shape(&e.category, big);
                        let svg_class = if mine { format!("{size} overflow-visible stroke-accent [stroke-width:1.5]") } else { format!("{size} overflow-visible") };
                        view! {
                            <button
                                type="button"
                                class=class
                                style=format!("left: {left:.2}%")
                                aria-label=format!("{label}{}", if mine { " (you)" } else { "" })
                                aria-pressed=move || (selected.get() == Some(idx)).to_string()
                                data-event-category=category_word(&e.category)
                                on:click=move |_| selected.update(|s| *s = if *s == Some(idx) { None } else { Some(idx) })
                            >
                                <svg viewBox="0 0 16 16" class=svg_class aria-hidden="true">{shape}</svg>
                            </button>
                        }
                    }).collect_view().into_any()
                }}
            </div>
            <ul class="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-caption text-muted" aria-label="Timeline legend">
                {CATEGORIES.iter().map(|c| view! {
                    <li class="inline-flex items-center gap-1.5">
                        <svg viewBox="0 0 16 16" class="size-3 text-secondary" aria-hidden="true">{marker_shape(c, false)}</svg>
                        {category_word(c)}
                    </li>
                }).collect_view()}
                <li class="inline-flex items-center gap-1.5"><SideMarker side=DraftSide::Blue text=SideText::Short /></li>
                <li class="inline-flex items-center gap-1.5"><SideMarker side=DraftSide::Red text=SideText::Short /></li>
                <li>"Chalk outline: you were involved"</li>
            </ul>
            {move || selected.get().and_then(|idx| events.get_value().get(idx).cloned()).map(|e| {
                let parts = participants.get_value();
                let text = event_tooltip(&e, &parts);
                let names: Vec<String> = e.involved_participants.iter()
                    .filter_map(|id| parts.iter().find(|p| p.participant_id == *id))
                    .map(|p| p.summoner_name.clone())
                    .collect();
                let tag_hint = match e.event_type.as_str() {
                    "ELITE_MONSTER_KILL" | "BUILDING_KILL" => "Objective+Control",
                    "CHAMPION_KILL" => "Teamfighting",
                    _ => "",
                };
                let href = format!(
                    "/personal-learnings/new?champion={}&opponent={}&match_id={}&result={}&event_ts={}&event_name={}&tag_hint={}",
                    user_champion, opponent_champion, match_id, if user_win { "win" } else { "loss" },
                    e.timestamp_ms, text.replace(' ', "+"), tag_hint
                );
                view! {
                    <div class="mt-4 border-t border-rule pt-3" aria-live="polite" data-timeline-detail>
                        <p class="text-body text-primary">{text}</p>
                        {(!names.is_empty()).then(|| view! { <p class="mt-1 text-label text-muted">{format!("Involved: {}", names.join(", "))}</p> })}
                        <div class="mt-2"><Button size=ButtonSize::Sm href=href>"Write a learning about this moment"</Button></div>
                    </div>
                }
            })}
        </section>
    }
}

// ---------------------------------------------------------------------------
// Performance
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
    (user_val / max * 100.0).clamp(0.0, 100.0)
}

fn avg_marker_pct(user_val: f32, avg: f32) -> f32 {
    let max = (user_val).max(avg * 2.0).max(1.0);
    (avg / max * 100.0).clamp(0.0, 100.0)
}

#[component]
fn PerformanceBar(
    label: &'static str,
    user_val: f32,
    avg: f32,
    display_val: String,
    against: &'static str,
) -> impl IntoView {
    let fill = normalize_pct(user_val, avg);
    let marker = avg_marker_pct(user_val, avg);
    let verdict = performance_verdict(user_val, avg);
    let verdict_class = match verdict {
        "Above average" => "text-label text-success",
        _ => "text-label text-muted",
    };
    view! {
        <div class="grid grid-cols-[8rem_minmax(0,1fr)_4rem] items-center gap-3 py-1.5 sm:grid-cols-[9rem_minmax(0,1fr)_4rem_8rem]">
            <span class="text-label text-secondary">{label}</span>
            <span class="relative h-3 border border-rule bg-ink-2" role="img" aria-label=format!("{label}: {display_val}, {verdict} against {against}")>
                <span class="block h-full bg-secondary" style=format!("width: {fill:.1}%")></span>
                <span class="absolute top-[-3px] h-[calc(100%+6px)] w-0.5 bg-primary" style=format!("left: {marker:.1}%") aria-hidden="true"></span>
            </span>
            <span class="figures text-right text-label font-semibold text-primary">{display_val.clone()}</span>
            <span class=format!("hidden sm:inline {verdict_class}")>{verdict}</span>
        </div>
    }
}

#[component]
fn PerformanceSection(perf: PerformanceStats) -> impl IntoView {
    let has_lane = perf.lane_opponent_damage.is_some();
    let mode = RwSignal::new("game".to_string());
    let perf = StoredValue::new(perf);
    view! {
        <section aria-labelledby="perf-h" data-performance>
            <SectionHeader
                title="Your performance"
                id="perf-h"
                actions=move || has_lane.then(|| view! {
                    <SegmentedControl id="perf-compare" label="Compare with" options=vec![Segment::new("game", "Game average"), Segment::new("lane", "Lane opponent")] value=mode size=SegmentSize::Sm />
                })
            />
            {(!has_lane).then(|| view! { <p class="mb-3 text-caption text-muted">"Compared with the game average; this game has no lane data for a lane comparison."</p> })}
            {move || perf.with_value(|p| {
                let lane = mode.get() == "lane" && has_lane;
                let against = if lane { "your lane opponent" } else { "the game average" };
                let vision_avg = if lane { p.lane_opponent_vision.unwrap_or(0) as f32 } else { p.vision_score_avg };
                let cs_avg = if lane { p.lane_opponent_cs_per_min.unwrap_or(0.0) } else { p.cs_per_min_avg };
                let gold_avg = if lane { p.lane_opponent_gold.unwrap_or(0) as f32 } else { p.gold_earned_avg };
                view! {
                    <div>
                        <PerformanceBar label="Damage share" user_val=p.damage_share_pct avg=10.0 display_val=format!("{:.1}%", p.damage_share_pct) against="an even share" />
                        <PerformanceBar label="Vision score" user_val=p.vision_score as f32 avg=vision_avg display_val=p.vision_score.to_string() against=against />
                        <PerformanceBar label="CS per minute" user_val=p.cs_per_min avg=cs_avg display_val=format!("{:.1}", p.cs_per_min) against=against />
                        <PerformanceBar label="Gold earned" user_val=p.gold_earned as f32 avg=gold_avg display_val=format_gold(p.gold_earned) against=against />
                    </div>
                }
            })}
            <p class="mt-2 text-caption text-muted">"The bar is your value; the thin line marks the comparison."</p>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

#[component]
pub fn MatchDetailPage() -> impl IntoView {
    use leptos_router::hooks::use_params_map;

    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
    let params = use_params_map();
    let match_id = move || params.read().get("id").unwrap_or_default();
    let detail = Resource::new(match_id, fetch_match_detail);

    view! {
        <PageShell attr:data-feedback-label="Match detail">
            <Suspense fallback=|| view! {
                <SkeletonGroup label="Loading the match">
                    <Skeleton width="w-64 max-w-full" size="h-10" />
                    <Skeleton shape=SkeletonShape::Block size="h-48" />
                    <Skeleton shape=SkeletonShape::Block size="h-48" />
                </SkeletonGroup>
            }>
                {move || detail.get().map(|r| match r {
                    Err(e) => view! {
                        <PageHeader title="Match" eyebrow="Match detail" />
                        <Callout
                            tone=CalloutTone::Warning
                            title="This match could not be loaded"
                            action=move || view! { <Button size=ButtonSize::Sm on_click=Callback::new(move |_| detail.refetch())>"Try again"</Button> }
                        >
                            {crate::models::utils::error_text(&e)}
                        </Callout>
                    }.into_any(),
                    Ok(d) => view! { <MatchBody d=d /> }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn MatchBody(d: MatchDetail) -> impl IntoView {
    let user = d
        .participants
        .iter()
        .find(|p| p.participant_id == d.user_participant_id)
        .cloned();
    let user_champion = user
        .as_ref()
        .map(|p| p.champion_name.clone())
        .unwrap_or_default();
    let user_position = user
        .as_ref()
        .map(|p| p.team_position.clone())
        .unwrap_or_default();
    let user_team = user.as_ref().map(|p| p.team_id).unwrap_or(100);
    let user_win = user.as_ref().map(|p| p.win).unwrap_or(false);
    let opponent_team = if user_team == 100 { 200 } else { 100 };
    let opponent_champion = if user_position.is_empty() {
        String::new()
    } else {
        d.participants
            .iter()
            .find(|p| p.team_id == opponent_team && p.team_position == user_position)
            .map(|p| p.champion_name.clone())
            .unwrap_or_default()
    };
    let blue: Vec<MatchParticipant> = d
        .participants
        .iter()
        .filter(|p| p.team_id == 100)
        .cloned()
        .collect();
    let red: Vec<MatchParticipant> = d
        .participants
        .iter()
        .filter(|p| p.team_id == 200)
        .cloned()
        .collect();
    let blue_win = blue.first().map(|p| p.win).unwrap_or(false);
    let red_win = red.first().map(|p| p.win).unwrap_or(false);
    let kda = user
        .as_ref()
        .map(|p| format!("{} / {} / {}", p.kills, p.deaths, p.assists))
        .unwrap_or_default();
    let learning_href = format!(
        "/personal-learnings/new?champion={}&opponent={}&match_id={}&result={}",
        user_champion,
        opponent_champion,
        d.match_id,
        if user_win { "win" } else { "loss" }
    );
    let title = if user_champion.is_empty() {
        "Match".to_string()
    } else {
        user_champion.clone()
    };
    let mode = d.game_mode.clone();
    let dur = format_duration(d.game_duration);
    let mid = d.match_id.clone();
    let has_user = user.is_some();
    view! {
        <PageHeader
            title=title
            eyebrow="Match detail"
            meta=move || view! {
                {has_user.then(|| view! { <ResultGlyph result=GameResult::from_win(user_win) word=true /> })}
                {(!kda.is_empty()).then(|| view! { <span class="figures">{kda.clone()}</span> })}
                <span>{format!("{mode} \u{00b7} {dur}")}</span>
                <span class="figures text-muted">{mid.clone()}</span>
            }
            actions=move || has_user.then(|| view! {
                <Button variant=ButtonVariant::Primary href=learning_href.clone()>"Write a learning"</Button>
            })
        />
        <div class="flex flex-col gap-12">
            <div class="grid gap-x-10 gap-y-10 xl:grid-cols-2">
                <TeamScoreboard side=DraftSide::Blue team_win=blue_win participants=blue user_participant_id=d.user_participant_id />
                <TeamScoreboard side=DraftSide::Red team_win=red_win participants=red user_participant_id=d.user_participant_id />
            </div>
            <Timeline d=d.clone() user_champion=user_champion.clone() opponent_champion=opponent_champion.clone() user_win=user_win />
            {has_user.then(|| view! { <PerformanceSection perf=d.performance.clone() /> })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdicts_and_bars() {
        assert_eq!(performance_verdict(12.0, 10.0), "Above average");
        assert_eq!(performance_verdict(8.0, 10.0), "Below average");
        assert_eq!(performance_verdict(10.0, 0.0), "Average");
        assert!((normalize_pct(10.0, 10.0) - 50.0).abs() < 0.01);
        assert_eq!(timeline_pct(900_000, 1800), 50.0);
        assert_eq!(position_role("UTILITY"), Some(Role::Support));
    }
}
