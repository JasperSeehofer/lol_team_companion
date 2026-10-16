//! Team games: every stored game the team's members played, grouped by
//! game, with a summary per member. Members sync their histories here
//! (one team sync a minute, within the shared Riot budget); personal
//! history, LP and champion trends are on My games.

use crate::components::local_time::LocalTime;
use crate::components::ui::{
    AvatarSize, Button, ButtonVariant, Callout, CalloutTone, ChampionAvatar, EmptyState,
    GameResult, GlyphSize, NoTeamState, PageHeader, PageShell, ReadoutSize, ResultGlyph,
    SectionHeader, Segment, SegmentSize, SegmentedControl, Skeleton, SkeletonGroup, SortState,
    SortStatus, SortTh, StatReadout, Table, Td, Toast, ToastContext, Tr,
};
use leptos::prelude::*;
use leptos_router::components::A;
use std::collections::HashMap;

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
    use crate::server::authz;
    Ok(authz::riot()?.is_configured())
}

#[server]
pub async fn get_team_stats() -> Result<Vec<TeamMatchRow>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    let rows = db::get_team_match_stats(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;

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
    use crate::server::authz;
    use crate::server::db;
    use crate::server::rate_limit::Bucket;
    use crate::server::riot;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;
    let riot_client = authz::riot()?;

    if !riot_client.is_configured() {
        return Err(ServerFnError::new(
            "RIOT_API_KEY not configured. Set it in .env to enable match syncing.",
        ));
    }
    authz::rate_limit(Bucket::Riot).await?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Err(ServerFnError::new("Create or join your team first")),
    };
    let members = db::get_roster_puuids(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;
    let team_region = db::get_team_region(&surreal, &team_id)
        .await
        .map_err(authz::server_err)?;

    // Fail fast when the global Riot budget cannot cover this sync.
    let linked = members
        .iter()
        .filter(|m| m.riot_puuid.as_deref().is_some_and(|p| !p.is_empty()))
        .count() as u32;
    riot_client
        .ensure_headroom(linked * riot::MATCH_HISTORY_REQUESTS)
        .map_err(authz::server_err)?;
    // A team sync makes ~21 Riot calls per linked player: once a minute per
    // team (not per member, so five members cannot run five syncs). Taken
    // after the budget check so a "Riot is busy" answer does not lock the
    // team out as well.
    authz::team_cooldown("team_sync", &team_id, std::time::Duration::from_secs(60))?;

    let queue_label = match queue_id {
        Some(420) => "Solo/Duo",
        Some(440) => "Flex",
        _ => "All Queues",
    };

    let mut total_synced = 0usize;
    let mut synced_players = 0usize;
    let mut errors = Vec::new();

    for member in &members {
        let (uid, username) = (&member.user_id, &member.username);
        let puuid = match &member.riot_puuid {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };

        // Each player's own profile region, else the team's.
        let platform = riot::platform_for(&[member.riot_region.as_deref(), team_region.as_deref()]);
        match riot_client
            .fetch_match_history(puuid, queue_id, platform)
            .await
        {
            Ok(matches) => {
                let count = matches.len();
                if let Err(e) = db::store_matches(&surreal, uid, matches).await {
                    tracing::error!(user = %uid, error = %e, "storing synced matches failed");
                    errors.push(format!("{username}: could not save matches"));
                } else {
                    total_synced += count;
                    synced_players += 1;
                }
            }
            Err(riot::RiotError::Busy(secs)) => {
                errors.push(format!("Riot is busy, try again in {secs} s"));
                break;
            }
            Err(e) => {
                tracing::warn!(user = %uid, error = %e, "Riot match history fetch failed");
                errors.push(format!("{username}: Riot API request failed"));
            }
        }
    }

    let linked_count = linked;
    let mut msg = format!("Synced {total_synced} {queue_label} matches for {synced_players}/{linked_count} linked players.");
    if !errors.is_empty() {
        msg.push_str(&format!(" Errors: {}", errors.join("; ")));
    }
    Ok(msg)
}

/// One game with the members who played it.
#[derive(Clone, Debug)]
pub struct MatchGroup {
    pub riot_match_id: String,
    pub queue_id: i32,
    pub game_end: Option<String>,
    pub game_duration: i32,
    pub win: bool,
    pub players: Vec<TeamMatchRow>,
}

/// Group rows by game, newest first.
pub fn group_matches(rows: &[TeamMatchRow]) -> Vec<MatchGroup> {
    let mut order: Vec<String> = Vec::new();
    let mut by_match: HashMap<String, Vec<TeamMatchRow>> = HashMap::new();
    for r in rows {
        if !by_match.contains_key(&r.riot_match_id) {
            order.push(r.riot_match_id.clone());
        }
        by_match
            .entry(r.riot_match_id.clone())
            .or_default()
            .push(r.clone());
    }
    let mut groups: Vec<MatchGroup> = order
        .into_iter()
        .filter_map(|id| {
            let players = by_match.remove(&id)?;
            let first = players.first()?.clone();
            Some(MatchGroup {
                riot_match_id: id,
                queue_id: first.queue_id,
                game_end: first.game_end.clone(),
                game_duration: first.game_duration,
                win: first.win,
                players,
            })
        })
        .collect();
    groups.sort_by(|a, b| b.game_end.cmp(&a.game_end));
    groups
}

/// Per member: (username, games, wins, kills, deaths, assists).
/// Sort keys of the games-by-member table (M141, report rucyy6k1).
const MEMBER_SORT: &[(&str, &str)] = &[
    ("member", "Member"),
    ("games", "Games"),
    ("winrate", "Win rate"),
    ("kda", "KDA"),
];

/// Member rows (name, games, wins, kills, deaths, assists) in the table's
/// order, stably; unsorted is the summary's own order.
pub fn sort_members(rows: &mut [(String, u32, u32, i32, i32, i32)], sort: SortState) {
    let Some(key) = sort.key else { return };
    let rate = |r: &(String, u32, u32, i32, i32, i32)| r.2 as f32 / r.1.max(1) as f32;
    let kda = |r: &(String, u32, u32, i32, i32, i32)| (r.3 + r.5) as f32 / r.4.max(1) as f32;
    crate::components::ui::sort_rows(rows, sort.dir, |a, b| match key {
        "member" => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        "games" => a.1.cmp(&b.1),
        "winrate" => rate(a).total_cmp(&rate(b)),
        "kda" => kda(a).total_cmp(&kda(b)),
        _ => std::cmp::Ordering::Equal,
    });
}

pub fn member_summary(rows: &[TeamMatchRow]) -> Vec<(String, u32, u32, i32, i32, i32)> {
    let mut out: Vec<(String, u32, u32, i32, i32, i32)> = Vec::new();
    for r in rows {
        match out.iter_mut().find(|e| e.0 == r.username) {
            Some(e) => {
                e.1 += 1;
                e.2 += r.win as u32;
                e.3 += r.kills;
                e.4 += r.deaths;
                e.5 += r.assists;
            }
            None => out.push((
                r.username.clone(),
                1,
                r.win as u32,
                r.kills,
                r.deaths,
                r.assists,
            )),
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out
}

fn format_duration(secs: i32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn queue_label(queue_id: i32) -> &'static str {
    match queue_id {
        420 => "Solo/Duo",
        440 => "Flex",
        450 => "ARAM",
        400 => "Normal draft",
        430 => "Normal blind",
        490 => "Quickplay",
        700 => "Clash",
        _ => "Other queue",
    }
}

#[component]
pub fn StatsPage() -> impl IntoView {
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
    let queue = RwSignal::new("all".to_string());
    let syncing = RwSignal::new(false);
    let sync = Callback::new(move |_| {
        let q = match queue.get_untracked().as_str() {
            "420" => Some(420),
            "440" => Some(440),
            _ => None,
        };
        syncing.set(true);
        leptos::task::spawn_local(async move {
            match sync_team_stats(q).await {
                Ok(msg) => {
                    toast.push.run(Toast::success(msg));
                    stats.refetch();
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(crate::models::utils::error_text(&e))),
            }
            syncing.set(false);
        });
    });

    view! {
        <PageShell attr:data-feedback-label="Team games">
            <PageHeader
                title="Team games"
                eyebrow="Review"
                meta=move || view! { <span>"Games your members played, from their synced Riot histories. Your own history, LP and champion trends are on My games."</span> }
                actions=move || view! {
                    <SegmentedControl id="sync-queue" label="Queue to sync" options=vec![Segment::new("all", "All queues"), Segment::new("420", "Solo/Duo"), Segment::new("440", "Flex")] value=queue size=SegmentSize::Sm />
                    <Button variant=ButtonVariant::Primary on_click=sync loading=syncing attr:data-testid="sync-team-games">"Sync team games"</Button>
                }
            />
            <p class="-mt-3 mb-6 max-w-prose text-caption text-muted">
                "A team sync reads the last games of every member with a linked Riot account (about 21 requests each). It runs once a minute per team, and the server's shared Riot budget can ask you to wait a few seconds."
            </p>
            <Suspense fallback=|| ()>
                {move || api_key.get().map(|r| (!r.unwrap_or(false)).then(|| view! {
                    <div class="mb-8"><Callout tone=CalloutTone::Info title="Riot features are off">"This server has no Riot API key, so games cannot be synced. Games stored earlier still show."</Callout></div>
                }))}
            </Suspense>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading team games"><Skeleton lines=6 /></SkeletonGroup> }>
                {move || stats.get().map(|r| match r {
                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"The team's games did not load. Reload the page in a moment."</Callout> }.into_any(),
                    Ok(rows) if rows.is_empty() => {
                        if matches!(has_team.get(), Some(false)) {
                            view! { <NoTeamState /> }.into_any()
                        } else {
                            view! { <EmptyState title="No games stored yet" message="Members link their Riot account on their profile; then a team sync stores their recent games here, grouped by game, with a summary per member." /> }.into_any()
                        }
                    }
                    Ok(rows) => view! { <StatsBody rows=rows /> }.into_any(),
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn StatsBody(rows: Vec<TeamMatchRow>) -> impl IntoView {
    let groups = group_matches(&rows);
    let members = member_summary(&rows);
    let member_sort = RwSignal::new(SortState::none());
    let games = groups.len();
    let wins = groups.iter().filter(|g| g.win).count();
    let rate = if games > 0 {
        format!("{:.0}", wins as f32 * 100.0 / games as f32)
    } else {
        "0".into()
    };
    let together = groups.iter().filter(|g| g.players.len() >= 2).count();
    view! {
        <div class="mb-10 flex flex-wrap gap-x-12 gap-y-6">
            <StatReadout label="Games" value=games.to_string() size=ReadoutSize::Md />
            <StatReadout label="Won" value=wins.to_string() size=ReadoutSize::Md />
            <StatReadout label="Win rate" value=rate unit="%" size=ReadoutSize::Md />
            <StatReadout label="Played together" value=together.to_string() size=ReadoutSize::Md />
        </div>
        <div class="grid gap-x-12 gap-y-12 lg:grid-cols-[minmax(0,1fr)_22rem]">
            <section aria-labelledby="games-list-h" class="min-w-0" data-team-games>
                <SectionHeader title="Games" id="games-list-h" meta="Newest first; a game shows every member who played it." />
                <ul class="border-b border-rule">
                    {groups.into_iter().map(|g| {
                        let href = format!("/match/{}", g.riot_match_id);
                        view! {
                            <li class="grid grid-cols-[1.5rem_minmax(0,1fr)] gap-x-3 gap-y-1 border-t border-rule py-3 sm:grid-cols-[1.5rem_12rem_minmax(0,1fr)]">
                                <span class="pt-0.5"><ResultGlyph result=GameResult::from_win(g.win) size=GlyphSize::Sm /></span>
                                <span class="text-label text-secondary">
                                    <A href=href attr:class="font-medium text-primary underline decoration-outline underline-offset-4 hover:decoration-accent">{queue_label(g.queue_id)}</A>
                                    <span class="figures block text-caption text-muted">
                                        {g.game_end.clone().map(|d| view! { <LocalTime iso=d /> })}
                                        {format!(" \u{00b7} {}", format_duration(g.game_duration))}
                                    </span>
                                </span>
                                <span class="col-start-2 flex flex-wrap gap-x-4 gap-y-1 sm:col-start-3">
                                    {g.players.into_iter().map(|p| view! {
                                        <span class="inline-flex items-center gap-1.5 text-label text-primary">
                                            <ChampionAvatar name=p.champion.clone() size=AvatarSize::Xs decorative=true />
                                            <span>{format!("{} ({})", p.username, p.champion)}</span>
                                            <span class="figures text-muted">{format!("{}/{}/{}", p.kills, p.deaths, p.assists)}</span>
                                        </span>
                                    }).collect_view()}
                                </span>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </section>
            <section aria-labelledby="members-sum-h" data-member-summary>
                <SectionHeader title="By member" id="members-sum-h" />
                <SortStatus sort=member_sort labels=MEMBER_SORT />
                <Table caption="Games by member" caption_hidden=true>
                    <thead>
                        <tr>
                            <SortTh label="Member" key="member" sort=member_sort />
                            <SortTh label="Games" key="games" sort=member_sort numeric=true />
                            <SortTh label="Win rate" key="winrate" sort=member_sort numeric=true />
                            <SortTh label="KDA" key="kda" sort=member_sort numeric=true />
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let mut rows = members.clone();
                            sort_members(&mut rows, member_sort.get());
                            rows.into_iter().map(|(name, games, wins, k, d, a)| view! {
                                <Tr>
                                    <Td>{name}</Td>
                                    <Td numeric=true>{games}</Td>
                                    <Td numeric=true>{format!("{:.0}%", wins as f32 * 100.0 / games.max(1) as f32)}</Td>
                                    <Td numeric=true>{format!("{:.1}", (k + a) as f32 / d.max(1) as f32)}</Td>
                                </Tr>
                            }).collect_view()
                        }}
                    </tbody>
                </Table>
            </section>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(game: &str, user: &str, end: &str, win: bool) -> TeamMatchRow {
        TeamMatchRow {
            match_db_id: format!("match:{game}"),
            user_id: format!("user:{user}"),
            username: user.into(),
            riot_match_id: game.into(),
            queue_id: 420,
            game_duration: 1805,
            game_end: Some(end.into()),
            champion: "Jinx".into(),
            kills: 2,
            deaths: 1,
            assists: 4,
            cs: 0,
            vision_score: 0,
            damage: 0,
            win,
        }
    }

    #[test]
    fn games_group_newest_first_with_every_member() {
        let rows = vec![
            row("A", "ann", "2026-10-01T00:00:00Z", true),
            row("B", "ann", "2026-10-03T00:00:00Z", false),
            row("A", "bo", "2026-10-01T00:00:00Z", true),
        ];
        let g = group_matches(&rows);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].riot_match_id, "B");
        assert_eq!(g[1].players.len(), 2);
        let m = member_summary(&rows);
        assert_eq!(m[0], ("ann".to_string(), 2, 1, 4, 2, 8));
        assert_eq!(format_duration(1805), "30:05");
    }
}
