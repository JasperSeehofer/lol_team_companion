//! My games: the personal layer's game page (it replaces the old solo
//! dashboard). Ranked standing, goals, the latest synced games with links to
//! the match detail, LP history and champion trends with date filters. All
//! from the user's own synced games; syncing goes through the shared Riot
//! budget and a one-minute cooldown, and the page says so.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::local_time::{LocalTime, TimeStyle};
use crate::components::ui::{
    AvatarSize, Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Callout, CalloutTone,
    ChampionAvatar, EmptySize, EmptyState, Field, GameResult, GlyphSize, InputKind, PageHeader,
    PageShell, ReadoutSize, ResultGlyph, SectionHeader, Segment, SegmentSize, SegmentedControl,
    Select, SelectOption, Skeleton, SkeletonGroup, StatReadout, Table, Td, TextField, Th, Toast,
    ToastContext, Tr,
};
use crate::models::match_data::{
    ChampionTrend, GoalProgress, GoalProgressPayload, PlayerMatchStats, RankedSnapshot,
};
use crate::models::user::RankedInfo;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MyGames {
    pub riot_id: Option<String>,
    pub has_puuid: bool,
    pub riot_configured: bool,
    pub ranked: Option<RankedInfo>,
    pub matches: Vec<PlayerMatchStats>,
}

#[server]
pub async fn get_my_games() -> Result<MyGames, ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let ranked = db::get_latest_ranked_snapshot(&ctx.db, &ctx.user.id, "RANKED_SOLO_5x5")
        .await
        .map_err(authz::server_err)?;
    let matches = db::get_solo_matches(&ctx.db, &ctx.user.id, None, 20)
        .await
        .map_err(authz::server_err)?;
    Ok(MyGames {
        riot_id: ctx.user.riot_summoner_name.clone(),
        has_puuid: ctx
            .user
            .riot_puuid
            .as_deref()
            .is_some_and(|p| !p.is_empty()),
        riot_configured: authz::riot().is_ok_and(|r| r.is_configured()),
        ranked,
        matches,
    })
}

/// Sync the user's last games and ranked standing from Riot. Rate limited
/// per IP, checked against the shared budget, then a one-minute cooldown.
#[server]
pub async fn sync_my_games() -> Result<i32, ServerFnError> {
    use crate::server::{authz, db, riot};

    let ctx = authz::authed().await?;
    let user = &ctx.user;
    let puuid = user
        .riot_puuid
        .as_deref()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| ServerFnError::new("Link your Riot account on your profile first."))?
        .to_string();
    authz::rate_limit(crate::server::rate_limit::Bucket::Riot).await?;
    let riot_client = authz::riot()?;
    riot_client
        .ensure_headroom(riot::MATCH_HISTORY_REQUESTS + 1)
        .map_err(authz::server_err)?;
    // After the budget check, so "Riot is busy" does not also start the cooldown.
    authz::user_cooldown("solo_sync", user, std::time::Duration::from_secs(60))?;
    let platform = riot::platform_for(&[user.riot_region.as_deref()]);
    let matches = riot_client
        .fetch_match_history(&puuid, None, platform)
        .await
        .map_err(authz::server_err)?;
    let count = matches.len();
    db::store_matches_with_synced_by(&ctx.db, &user.id, matches, Some(&user.id))
        .await
        .map_err(authz::server_err)?;
    let ranked = riot_client
        .fetch_ranked_data(&puuid, platform)
        .await
        .map_err(authz::server_err)?;
    for e in ranked {
        if e.queue_type.contains("RANKED_SOLO") || e.queue_type.contains("RANKED_FLEX") {
            db::store_ranked_snapshot(
                &ctx.db,
                &user.id,
                &e.queue_type,
                &e.tier,
                &e.division,
                e.lp,
                e.wins,
                e.losses,
            )
            .await
            .map_err(authz::server_err)?;
        }
    }
    db::update_last_solo_sync(&ctx.db, &user.id)
        .await
        .map_err(authz::server_err)?;
    Ok(count as i32)
}

/// The date filters of the LP history and champion trends.
pub const WINDOWS: [(&str, &str); 4] = [
    ("7d", "7 days"),
    ("30d", "30 days"),
    ("90d", "90 days"),
    ("all", "All"),
];

/// The cutoff (RFC 3339) for a window key; `None` is all time.
pub fn window_cutoff(window: &str, now: chrono::DateTime<chrono::Utc>) -> Option<String> {
    let days: i64 = match window {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => return None,
    };
    Some((now - chrono::Duration::days(days)).to_rfc3339())
}

#[server]
pub async fn get_my_lp_history(window: String) -> Result<Vec<RankedSnapshot>, ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    db::get_lp_history(
        &ctx.db,
        &ctx.user.id,
        window_cutoff(&window, chrono::Utc::now()),
    )
    .await
    .map_err(authz::server_err)
}

#[server]
pub async fn get_my_champion_trends(window: String) -> Result<Vec<ChampionTrend>, ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    db::get_champion_trends(
        &ctx.db,
        &ctx.user.id,
        window_cutoff(&window, chrono::Utc::now()),
    )
    .await
    .map_err(authz::server_err)
}

#[server]
pub async fn get_my_goals() -> Result<GoalProgressPayload, ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    db::compute_goal_progress(&ctx.db, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

const TIERS: [&str; 10] = [
    "IRON",
    "BRONZE",
    "SILVER",
    "GOLD",
    "PLATINUM",
    "EMERALD",
    "DIAMOND",
    "MASTER",
    "GRANDMASTER",
    "CHALLENGER",
];
const DIVISIONS: [&str; 4] = ["IV", "III", "II", "I"];

#[server]
pub async fn set_my_goal(goal_type: String, target_value: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let target = target_value.trim().to_string();
    match goal_type.as_str() {
        "cs_per_min" => {
            let v: f32 = target
                .parse()
                .map_err(|_| ServerFnError::new("CS per minute is a number, like 7.5."))?;
            if !(0.0..=15.0).contains(&v) {
                return Err(ServerFnError::new("CS per minute is between 0 and 15."));
            }
        }
        "deaths_per_game" => {
            let v: i32 = target
                .parse()
                .map_err(|_| ServerFnError::new("Deaths per game is a whole number."))?;
            if !(0..=20).contains(&v) {
                return Err(ServerFnError::new("Deaths per game is between 0 and 20."));
            }
        }
        "rank_target" => {
            let (tier, div) = target.split_once(':').unwrap_or((target.as_str(), ""));
            if !TIERS.contains(&tier) || !(div.is_empty() || DIVISIONS.contains(&div)) {
                return Err(ServerFnError::new("Choose a tier and a division."));
            }
        }
        _ => return Err(ServerFnError::new("Unknown goal")),
    }
    db::upsert_personal_goal(&ctx.db, &ctx.user.id, &goal_type, &target)
        .await
        .map_err(authz::server_err)
}

fn tier_word(t: &str) -> String {
    let mut c = t.to_lowercase();
    if let Some(f) = c.get_mut(0..1) {
        f.make_ascii_uppercase();
    }
    c
}

/// "Gold II", "Master".
pub fn rank_words(tier: &str, division: &str) -> String {
    if matches!(tier, "MASTER" | "GRANDMASTER" | "CHALLENGER") || division.is_empty() {
        tier_word(tier)
    } else {
        format!("{} {division}", tier_word(tier))
    }
}

#[component]
pub fn MyGamesPage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
    let data = Resource::new(|| (), |_| get_my_games());
    // Every resource is created here, before any Suspense, so the server and
    // the browser register them in the same order.
    let lp_window = RwSignal::new("30d".to_string());
    let trends_window = RwSignal::new("30d".to_string());
    let res = MyResources {
        lp_window,
        history: Resource::new(move || lp_window.get(), get_my_lp_history),
        trends_window,
        trends: Resource::new(move || trends_window.get(), get_my_champion_trends),
        goals: Resource::new(|| (), |_| get_my_goals()),
    };
    let syncing = RwSignal::new(false);
    let sync = Callback::new(move |_| {
        syncing.set(true);
        leptos::task::spawn_local(async move {
            match sync_my_games().await {
                Ok(n) => {
                    toast.push.run(Toast::success(format!(
                        "Synced {n} games and your ranked standing"
                    )));
                    data.refetch();
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(crate::models::utils::error_text(&e))),
            }
            syncing.set(false);
        });
    });

    view! {
        <PageShell attr:data-feedback-label="My games">
            <PageHeader
                title="My games"
                eyebrow="You"
                meta=move || view! {
                    <Suspense fallback=|| ()>
                        {move || data.get().and_then(|r| r.ok()).map(|d| match d.riot_id {
                            Some(id) => view! { <span>{format!("Riot account {id}")}</span> }.into_any(),
                            None => view! { <span>"No Riot account linked"</span> }.into_any(),
                        })}
                    </Suspense>
                }
                actions=move || view! {
                    // Disabled until there is something to sync with (read in a Suspense).
                    <Suspense fallback=|| ()>
                        {move || data.get().map(|r| {
                            let ready = r.map(|d| d.has_puuid && d.riot_configured).unwrap_or(false);
                            view! { <Button variant=ButtonVariant::Primary on_click=sync loading=syncing disabled=!ready attr:data-testid="sync-my-games">"Sync my games"</Button> }
                        })}
                    </Suspense>
                }
            />
            <p class="-mt-3 mb-6 max-w-prose text-caption text-muted">
                "A sync reads your last 20 games and your ranked standing from Riot (about 21 requests). The server shares one request budget, so a sync can be refused for a minute; after a sync, the next one is possible a minute later."
            </p>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading your games"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || data.get().map(|r| match r {
                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"Your games did not load. Reload the page in a moment."</Callout> }.into_any(),
                    Ok(d) => view! { <MyGamesBody data=d res=res /> }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

/// The page's resources (see `MyGamesPage`).
#[derive(Clone, Copy)]
struct MyResources {
    lp_window: RwSignal<String>,
    history: Resource<Result<Vec<RankedSnapshot>, ServerFnError>>,
    trends_window: RwSignal<String>,
    trends: Resource<Result<Vec<ChampionTrend>, ServerFnError>>,
    goals: Resource<Result<GoalProgressPayload, ServerFnError>>,
}

#[component]
fn MyGamesBody(data: MyGames, res: MyResources) -> impl IntoView {
    let notice = if !data.riot_configured {
        Some(view! { <Callout tone=CalloutTone::Info title="Riot features are off">"This server has no Riot API key, so games cannot be synced. Games stored earlier still show."</Callout> }.into_any())
    } else if !data.has_puuid {
        Some(view! {
            <Callout tone=CalloutTone::Info title="No Riot account linked" action=move || view! { <Button size=ButtonSize::Sm href="/profile">"Link it on your profile"</Button> }>
                "Link your Riot account to sync your games, ranked standing and LP."
            </Callout>
        }.into_any())
    } else {
        None
    };
    view! {
        {notice.map(|n| view! { <div class="mb-8">{n}</div> })}
        <div class="grid gap-x-12 gap-y-12 lg:grid-cols-[minmax(0,1fr)_22rem]">
            <div class="flex min-w-0 flex-col gap-12">
                <RecentGames matches=data.matches.clone() />
                <LpHistory window=res.lp_window history=res.history />
                <ChampionTrends window=res.trends_window trends=res.trends />
            </div>
            <div class="flex min-w-0 flex-col gap-12">
                <Standing ranked=data.ranked.clone() />
                <Goals goals=res.goals />
            </div>
        </div>
    }
}

#[component]
fn Standing(ranked: Option<RankedInfo>) -> impl IntoView {
    view! {
        <section aria-labelledby="standing-h" data-standing>
            <SectionHeader title="Ranked standing" id="standing-h" meta="Solo/Duo" />
            {match ranked {
                None => view! { <p class="text-body text-secondary">"Your ranked standing appears after a sync, once you have played ranked Solo/Duo this season."</p> }.into_any(),
                Some(r) => {
                    let games = r.wins + r.losses;
                    let rate = if games > 0 { format!("{:.0}", r.wins as f32 * 100.0 / games as f32) } else { "0".into() };
                    view! {
                        <div class="grid grid-cols-2 gap-x-6 gap-y-5">
                            <StatReadout label="Rank" value=rank_words(&r.tier, &r.division) size=ReadoutSize::Md />
                            <StatReadout label="League points" value=r.lp.to_string() unit="LP" size=ReadoutSize::Md />
                            <StatReadout label="Record" value=format!("{}W {}L", r.wins, r.losses) size=ReadoutSize::Sm />
                            <StatReadout label="Win rate" value=rate unit="%" size=ReadoutSize::Sm />
                        </div>
                    }.into_any()
                }
            }}
        </section>
    }
}

#[component]
fn RecentGames(matches: Vec<PlayerMatchStats>) -> impl IntoView {
    view! {
        <section aria-labelledby="recent-h" data-my-recent>
            <SectionHeader title="Recent games" id="recent-h" meta="Your last 20 synced games" />
            {if matches.is_empty() {
                view! { <EmptyState size=EmptySize::Compact message="No games synced yet. Link your Riot account and sync; your last 20 games appear here, each with its full scoreboard." /> }.into_any()
            } else {
                view! {
                    <Table caption="Your recent games" caption_hidden=true>
                        <thead>
                            <tr>
                                <Th>"Result"</Th>
                                <Th>"Champion"</Th>
                                <Th numeric=true>"K / D / A"</Th>
                                <Th numeric=true>"CS"</Th>
                                <Th numeric=true>"Damage"</Th>
                                <Th>"Game"</Th>
                            </tr>
                        </thead>
                        <tbody>
                            {matches.into_iter().map(|m| {
                                let href = format!("/match/{}", m.match_id);
                                view! {
                                    <Tr>
                                        <Td><ResultGlyph result=GameResult::from_win(m.win) size=GlyphSize::Sm /></Td>
                                        <Td>
                                            <span class="flex items-center gap-2">
                                                <ChampionAvatar name=m.champion.clone() size=AvatarSize::Xs decorative=true />
                                                {m.champion.clone()}
                                            </span>
                                        </Td>
                                        <Td numeric=true>{format!("{} / {} / {}", m.kills, m.deaths, m.assists)}</Td>
                                        <Td numeric=true>{m.cs}</Td>
                                        <Td numeric=true>{format!("{:.1}k", m.damage as f32 / 1000.0)}</Td>
                                        <Td><A href=href attr:class="text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">"Details"</A></Td>
                                    </Tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </Table>
                }.into_any()
            }}
        </section>
    }
}

/// Points for the LP chart in a 600 x 160 box (x by time, y by rank score).
pub fn lp_points(snaps: &[RankedSnapshot]) -> Vec<(f32, f32)> {
    let times: Vec<i64> = snaps
        .iter()
        .map(|s| {
            crate::components::local_time::parse_utc(&s.snapshotted_at)
                .map(|t| t.timestamp())
                .unwrap_or(0)
        })
        .collect();
    let (t0, t1) = (
        times.iter().copied().min().unwrap_or(0),
        times.iter().copied().max().unwrap_or(0),
    );
    let (s0, s1) = (
        snaps.iter().map(|s| s.rank_score).min().unwrap_or(0),
        snaps.iter().map(|s| s.rank_score).max().unwrap_or(0),
    );
    let span_t = (t1 - t0).max(1) as f32;
    let span_s = (s1 - s0).max(1) as f32;
    snaps
        .iter()
        .zip(times)
        .map(|(s, t)| {
            let x = if t1 == t0 {
                300.0
            } else {
                (t - t0) as f32 / span_t * 600.0
            };
            let y = if s1 == s0 {
                80.0
            } else {
                150.0 - (s.rank_score - s0) as f32 / span_s * 140.0
            };
            (x, y)
        })
        .collect()
}

#[component]
fn LpHistory(
    window: RwSignal<String>,
    history: Resource<Result<Vec<RankedSnapshot>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="lp-h" data-lp-history>
            <SectionHeader
                title="LP history"
                id="lp-h"
                actions=move || view! {
                    <SegmentedControl id="lp-window" label="LP history period" options=WINDOWS.iter().map(|(k, l)| Segment::new(*k, *l)).collect() value=window size=SegmentSize::Sm />
                }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading LP history"><Skeleton lines=3 /></SkeletonGroup> }>
                {move || history.get().map(|r| {
                    let snaps = r.unwrap_or_default();
                    if snaps.len() < 2 {
                        return view! { <EmptyState size=EmptySize::Compact message="Not enough points in this period. Each sync stores your Solo/Duo standing; the line appears from two syncs on." /> }.into_any();
                    }
                    let pts = lp_points(&snaps);
                    let path = pts.iter().enumerate().map(|(i, (x, y))| format!("{}{x:.1},{y:.1}", if i == 0 { "M" } else { " L" })).collect::<String>();
                    let first = snaps.first().cloned();
                    let last = snaps.last().cloned();
                    let (first, last) = (first.unwrap_or_else(|| snaps[0].clone()), last.unwrap_or_else(|| snaps[0].clone()));
                    let summary = format!(
                        "LP history, {} points: from {} {} LP to {} {} LP",
                        snaps.len(),
                        rank_words(&first.tier, &first.division), first.lp,
                        rank_words(&last.tier, &last.division), last.lp,
                    );
                    view! {
                        <figure>
                            <svg viewBox="-8 -8 616 176" class="h-auto w-full text-secondary" role="img" aria-label=summary.clone()>
                                <line x1="0" y1="158" x2="600" y2="158" class="stroke-rule" stroke-width="1" />
                                <path d=path fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
                                {pts.iter().zip(snaps.iter()).map(|((x, y), s)| view! {
                                    <circle cx=format!("{x:.1}") cy=format!("{y:.1}") r="4" class="fill-ink-1" stroke="currentColor" stroke-width="2">
                                        <title>{format!("{} {} LP", rank_words(&s.tier, &s.division), s.lp)}</title>
                                    </circle>
                                }).collect_view()}
                            </svg>
                            <figcaption class="mt-2 flex justify-between gap-3 text-caption text-muted">
                                <span><LocalTime iso=first.snapshotted_at.clone() style=TimeStyle::DayMonth />{format!(": {} {} LP", rank_words(&first.tier, &first.division), first.lp)}</span>
                                <span><LocalTime iso=last.snapshotted_at.clone() style=TimeStyle::DayMonth />{format!(": {} {} LP", rank_words(&last.tier, &last.division), last.lp)}</span>
                            </figcaption>
                            <details class="mt-2 text-label text-secondary">
                                <summary class="cursor-pointer text-muted">"All points as a table"</summary>
                                <Table caption="LP history points" caption_hidden=true>
                                    <thead><tr><Th>"When"</Th><Th>"Rank"</Th><Th numeric=true>"LP"</Th></tr></thead>
                                    <tbody>
                                        {snaps.clone().into_iter().map(|s| view! {
                                            <Tr>
                                                <Td><LocalTime iso=s.snapshotted_at.clone() /></Td>
                                                <Td>{rank_words(&s.tier, &s.division)}</Td>
                                                <Td numeric=true>{s.lp}</Td>
                                            </Tr>
                                        }).collect_view()}
                                    </tbody>
                                </Table>
                            </details>
                        </figure>
                    }.into_any()
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn ChampionTrends(
    window: RwSignal<String>,
    trends: Resource<Result<Vec<ChampionTrend>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="trends-h" data-champion-trends>
            <SectionHeader
                title="Champion trends"
                id="trends-h"
                actions=move || view! {
                    <SegmentedControl id="trends-window" label="Champion trends period" options=WINDOWS.iter().map(|(k, l)| Segment::new(*k, *l)).collect() value=window size=SegmentSize::Sm />
                }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading champion trends"><Skeleton lines=3 /></SkeletonGroup> }>
                {move || trends.get().map(|r| {
                    let list = r.unwrap_or_default();
                    if list.is_empty() {
                        return view! { <EmptyState size=EmptySize::Compact message="No synced games in this period. Your champions appear here with games, win rate, KDA, CS per minute and damage." /> }.into_any();
                    }
                    view! {
                        <Table caption="Champion trends" caption_hidden=true>
                            <thead>
                                <tr>
                                    <Th>"Champion"</Th>
                                    <Th numeric=true>"Games"</Th>
                                    <Th numeric=true>"Win rate"</Th>
                                    <Th numeric=true>"KDA"</Th>
                                    <Th numeric=true>"CS / min"</Th>
                                    <Th numeric=true>"Damage"</Th>
                                </tr>
                            </thead>
                            <tbody>
                                {list.into_iter().map(|t| view! {
                                    <Tr>
                                        <Td>
                                            <span class="flex items-center gap-2">
                                                <ChampionAvatar name=t.champion.clone() size=AvatarSize::Xs decorative=true />
                                                {t.champion.clone()}
                                            </span>
                                        </Td>
                                        <Td numeric=true>{t.games}</Td>
                                        <Td numeric=true>{format!("{:.0}%", t.wins as f32 * 100.0 / t.games.max(1) as f32)}</Td>
                                        <Td numeric=true>{format!("{:.1}", t.avg_kda)}</Td>
                                        <Td numeric=true>{format!("{:.1}", t.cs_per_min)}</Td>
                                        <Td numeric=true>{format!("{:.1}k", t.avg_damage as f32 / 1000.0)}</Td>
                                    </Tr>
                                }).collect_view()}
                            </tbody>
                        </Table>
                    }.into_any()
                })}
            </Suspense>
        </section>
    }
}

/// Personal goals: a rank to reach, CS per minute and deaths per game, each
/// measured over the last ranked Solo/Duo games (from five games on).
#[component]
fn Goals(goals: Resource<Result<GoalProgressPayload, ServerFnError>>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let tier_options: Vec<SelectOption> = TIERS
        .iter()
        .map(|t| SelectOption::new(*t, tier_word(t)))
        .collect();
    let division_options: Vec<SelectOption> = DIVISIONS
        .iter()
        .map(|d| SelectOption::new(*d, *d))
        .collect();
    let tier_options = StoredValue::new(tier_options);
    let division_options = StoredValue::new(division_options);
    let tier = RwSignal::new("GOLD".to_string());
    let division = RwSignal::new("IV".to_string());
    let cs = RwSignal::new(String::new());
    let deaths = RwSignal::new(String::new());
    let save = Callback::new(move |(kind, value): (&'static str, String)| {
        leptos::task::spawn_local(async move {
            match set_my_goal(kind.to_string(), value).await {
                Ok(()) => {
                    toast.push.run(Toast::success("Goal saved"));
                    goals.refetch();
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(crate::models::utils::error_text(&e))),
            }
        });
    });
    let progress_line = |g: &Option<GoalProgress>, fmt: fn(f32) -> String| -> AnyView {
        match g {
            None => view! { <p class="text-label text-muted">"No goal set."</p> }.into_any(),
            Some(p) => {
                let target = p.goal.target_value.replace(':', " ");
                let now = match p.current_value {
                    Some(v) => format!("Now {} over your last {} games.", fmt(v), p.game_count),
                    None => format!("Needs 5 ranked Solo/Duo games ({} so far).", p.game_count),
                };
                view! {
                    <p class="flex flex-wrap items-center gap-2 text-label text-secondary">
                        <span>{format!("Target {target}.")}</span>
                        <span>{now}</span>
                        {p.achieved.then(|| view! { <Badge tone=BadgeTone::Success>"Reached"</Badge> })}
                    </p>
                }.into_any()
            }
        }
    };
    view! {
        <section aria-labelledby="goals-h" data-goals>
            <SectionHeader title="Goals" id="goals-h" />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading goals"><Skeleton lines=3 /></SkeletonGroup> }>
                {move || goals.get().map(|r| {
                    let g = r.unwrap_or(GoalProgressPayload { rank: None, cs: None, deaths: None, current_rank: None });
                    let current = g.current_rank.as_ref().map(|c| rank_words(&c.tier, &c.division));
                    view! {
                        <div class="flex flex-col gap-6">
                            <div class="border-t border-rule pt-3">
                                <h3 class="font-medium text-primary">"Rank"</h3>
                                {progress_line(&g.rank, |_| String::new())}
                                {current.map(|c| view! { <p class="text-caption text-muted">{format!("Current: {c}")}</p> })}
                                <div class="mt-2 flex flex-wrap items-end gap-2">
                                    <Field id="goal-tier" label="Tier">
                                        <Select value=tier options=tier_options.get_value() />
                                    </Field>
                                    <Field id="goal-division" label="Division">
                                        <Select value=division options=division_options.get_value() />
                                    </Field>
                                    <Button size=ButtonSize::Sm on_click=Callback::new(move |_| save.run(("rank_target", format!("{}:{}", tier.get_untracked(), division.get_untracked()))))>"Set rank goal"</Button>
                                </div>
                            </div>
                            <div class="border-t border-rule pt-3">
                                <h3 class="font-medium text-primary">"CS per minute"</h3>
                                {progress_line(&g.cs, |v| format!("{v:.1}"))}
                                <div class="mt-2 flex flex-wrap items-end gap-2">
                                    <Field id="goal-cs" label="Target">
                                        <TextField value=cs kind=InputKind::Number placeholder="7.5" size=crate::components::ui::FieldSize::Sm on_keydown=crate::components::ui::enter_submits(Callback::new(move |_| save.run(("cs_per_min", cs.get_untracked())))) />
                                    </Field>
                                    <Button size=ButtonSize::Sm on_click=Callback::new(move |_| save.run(("cs_per_min", cs.get_untracked())))>"Set CS goal"</Button>
                                </div>
                            </div>
                            <div class="border-t border-rule pt-3">
                                <h3 class="font-medium text-primary">"Deaths per game"</h3>
                                {progress_line(&g.deaths, |v| format!("{v:.1}"))}
                                <div class="mt-2 flex flex-wrap items-end gap-2">
                                    <Field id="goal-deaths" label="At most">
                                        <TextField value=deaths kind=InputKind::Number placeholder="4" size=crate::components::ui::FieldSize::Sm on_keydown=crate::components::ui::enter_submits(Callback::new(move |_| save.run(("deaths_per_game", deaths.get_untracked())))) />
                                    </Field>
                                    <Button size=ButtonSize::Sm on_click=Callback::new(move |_| save.run(("deaths_per_game", deaths.get_untracked())))>"Set deaths goal"</Button>
                                </div>
                            </div>
                        </div>
                    }
                })}
            </Suspense>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_cut_off_where_they_say() {
        let now = chrono::Utc::now();
        let c = window_cutoff("7d", now).unwrap();
        let t = crate::components::local_time::parse_utc(&c).unwrap();
        assert_eq!((now - t).num_days(), 7);
        assert!(window_cutoff("all", now).is_none());
        assert!(window_cutoff("bogus", now).is_none());
    }

    #[test]
    fn ranks_read_as_words() {
        assert_eq!(rank_words("GOLD", "II"), "Gold II");
        assert_eq!(rank_words("MASTER", "I"), "Master");
    }

    #[test]
    fn lp_points_span_the_box() {
        let s = |t: &str, score: i32| RankedSnapshot {
            id: None,
            tier: "GOLD".into(),
            division: "II".into(),
            lp: 0,
            snapshotted_at: t.into(),
            rank_score: score,
        };
        let pts = lp_points(&[
            s("2026-10-01T00:00:00Z", 100),
            s("2026-10-11T00:00:00Z", 200),
        ]);
        assert_eq!(pts[0], (0.0, 150.0));
        assert_eq!(pts[1], (600.0, 10.0));
    }
}
