//! The team page: the roster (starting five, substitutes on the bench,
//! coaches, members not placed yet) with the leader's tools, join requests,
//! team notes every member can edit with Undo, recent games and pool gaps.
//! Real data only; every empty part says how it fills.

use crate::components::local_time::{LocalTime, TimeStyle};
use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, ChampionAvatar, ConfirmDialog, ConfirmTone,
    Dialog, EmptySize, EmptyState, Field, GameResult, GlyphSize, Menu, MenuAlign, MenuItem,
    NoTeamState, PageHeader, PageShell, ResultGlyph, Role, RoleLabel, SectionHeader, Select,
    SelectOption, Skeleton, SkeletonGroup, SkeletonShape, TextField, Toast, ToastContext,
    UserAvatar,
};
use crate::models::game_plan::PoolGapWarning;
use crate::models::team::Team;
use crate::models::team_note::TeamNote;
use crate::models::user::{JoinRequest, TeamMember};
use leptos::prelude::*;
use leptos_router::components::A;

/// Returns (team, members, current_user_id) so the client can check leadership.
#[server]
pub async fn get_team_dashboard() -> Result<Option<(Team, Vec<TeamMember>, String)>, ServerFnError>
{
    use crate::server::authz;
    use crate::server::db;

    let user = authz::require_user().await?;
    let user_id = user.id.clone();
    let db = authz::db()?;

    match db::get_user_team_with_members(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some((team, members)) => Ok(Some((team, members, user_id))),
        None => Ok(None),
    }
}

#[server]
pub async fn update_team_info(name: String, region: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_leader().await?;
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(ServerFnError::new("A team name has 1 to 60 characters."));
    }
    if !TEAM_REGIONS.contains(&region.as_str()) {
        return Err(ServerFnError::new("Unknown region"));
    }
    db::update_team(&ctx.db, &team_id, name, region)
        .await
        .map_err(authz::server_err)
}

/// Place a member (leader only): `top` ... `support` starts them in that
/// position (the starter there moves to the bench), `substitute` benches
/// them, `coach` makes them a coach, `member` leaves them unplaced.
#[server]
pub async fn set_member_place(member_user_id: String, place: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db::{self, RosterPlace};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_leader().await?;
    let result = match place.as_str() {
        "substitute" => {
            db::set_member_place(&ctx.db, &team_id, &member_user_id, RosterPlace::Substitute).await
        }
        "coach" => {
            db::set_member_place(&ctx.db, &team_id, &member_user_id, RosterPlace::Coach).await
        }
        "member" => {
            db::set_member_place(&ctx.db, &team_id, &member_user_id, RosterPlace::Member).await
        }
        position => match Role::parse(position) {
            Some(r) if r.key() == position => {
                db::assign_to_slot(&ctx.db, &team_id, &member_user_id, position).await
            }
            _ => return Err(ServerFnError::new("Unknown roster place")),
        },
    };
    result.map_err(authz::server_err)
}

#[server]
pub async fn kick_member(member_user_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_leader().await?;
    if member_user_id == ctx.user.id {
        return Err(ServerFnError::new(
            "The team leader cannot remove themselves.",
        ));
    }
    db::remove_team_member(&ctx.db, &team_id, &member_user_id)
        .await
        .map_err(authz::server_err)
}

/// Make another member the team's leader (M8). The leader keeps their
/// membership; the new leader gets the leader's actions.
#[server]
pub async fn hand_over_leadership(member_user_id: String) -> Result<(), ServerFnError> {
    use crate::server::{account, authz};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_leader().await?;
    if member_user_id == ctx.user.id {
        return Err(ServerFnError::new("You already lead the team."));
    }
    account::hand_over_leadership(&ctx.db, &team_id, &member_user_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn leave_team() -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    if db::is_team_leader(&ctx.db, &ctx.user.id, &team_id)
        .await
        .map_err(authz::server_err)?
    {
        return Err(ServerFnError::new("The team leader cannot leave the team."));
    }
    db::remove_team_member(&ctx.db, &team_id, &ctx.user.id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_pending_requests() -> Result<Vec<JoinRequest>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    if !db::is_team_leader(&ctx.db, &ctx.user.id, &team_id)
        .await
        .map_err(authz::server_err)?
    {
        return Ok(Vec::new()); // non-leaders see an empty list
    }
    db::list_pending_join_requests(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn handle_join_request(request_id: String, accept: bool) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_leader().await?;
    db::respond_to_join_request(&ctx.db, &request_id, accept, &team_id)
        .await
        .map_err(authz::server_err)
}

/// Regions a team can play in.
pub const TEAM_REGIONS: &[&str] = &[
    "EUW", "EUNE", "NA", "KR", "BR", "LAN", "LAS", "OCE", "TR", "RU", "JP",
];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecentMatch {
    pub riot_match_id: String,
    pub champion: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub win: bool,
    pub game_end: Option<String>,
    pub username: String,
}

/// The team's five most recent stored games (one row per game; the first
/// member found in it).
#[server]
pub async fn get_recent_team_matches() -> Result<Vec<RecentMatch>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    let rows = db::get_team_match_stats(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let mut seen = std::collections::HashSet::new();
    Ok(rows
        .into_iter()
        .filter(|r| seen.insert(r.riot_match_id.clone()))
        .take(5)
        .map(|r| RecentMatch {
            riot_match_id: r.riot_match_id,
            champion: r.champion,
            kills: r.kills,
            deaths: r.deaths,
            assists: r.assists,
            win: r.win,
            game_end: r.game_end,
            username: r.username,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Team notes: every member may write, edit, pin and delete, with Undo
// ---------------------------------------------------------------------------

#[server]
pub async fn get_all_team_notes() -> Result<Vec<TeamNote>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    db::list_team_notes(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn add_team_note(content: String) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::create_team_note(&ctx.db, &team_id, &ctx.user.id, &ctx.user.username, content)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn toggle_note_pin(note_id: String, pinned: bool) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::toggle_pin_team_note(&ctx.db, &team_id, &note_id, pinned)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn edit_team_note(note_id: String, content: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::update_team_note(&ctx.db, &team_id, &note_id, content, &ctx.user.username)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn undo_note_edit(note_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::undo_team_note_edit(&ctx.db, &team_id, &note_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn remove_team_note(note_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::delete_team_note(&ctx.db, &team_id, &note_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn restore_note(note_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::restore_team_note(&ctx.db, &team_id, &note_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_pool_gap_panel() -> Result<Vec<PoolGapWarning>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    let summary = db::get_dashboard_summary(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    Ok(summary.pool_gap_warnings)
}

// ---------------------------------------------------------------------------
// Roster helpers
// ---------------------------------------------------------------------------

/// The roster split by place: starting five by position, substitutes,
/// coaches, not placed yet.
pub struct RosterGroups {
    pub starters: [Option<TeamMember>; 5],
    pub substitutes: Vec<TeamMember>,
    pub coaches: Vec<TeamMember>,
    pub unplaced: Vec<TeamMember>,
}

pub fn group_roster(members: &[TeamMember]) -> RosterGroups {
    let mut starters: [Option<TeamMember>; 5] = Default::default();
    let mut substitutes = Vec::new();
    let mut coaches = Vec::new();
    let mut unplaced = Vec::new();
    for m in members {
        match m.roster_type.as_str() {
            "starter" => match Role::parse(&m.role) {
                Some(r) => {
                    let i = Role::ALL.iter().position(|x| *x == r).unwrap_or(0);
                    if starters[i].is_none() {
                        starters[i] = Some(m.clone());
                    } else {
                        substitutes.push(m.clone());
                    }
                }
                None => unplaced.push(m.clone()),
            },
            "substitute" => substitutes.push(m.clone()),
            "coach" => coaches.push(m.clone()),
            _ => unplaced.push(m.clone()),
        }
    }
    for list in [&mut substitutes, &mut coaches, &mut unplaced] {
        list.sort_by_key(|m| m.username.to_lowercase());
    }
    RosterGroups {
        starters,
        substitutes,
        coaches,
        unplaced,
    }
}

/// A `&'static str` for a DOM id prefix (the `Menu` primitive takes one).
/// Each distinct string is leaked once, so the set stays as small as the
/// number of distinct members seen.
fn intern(s: String) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static SEEN: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut set = match SEEN.get_or_init(|| Mutex::new(HashSet::new())).lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    if let Some(found) = set.get(s.as_str()) {
        return found;
    }
    let leaked: &'static str = Box::leak(s.into_boxed_str());
    set.insert(leaked);
    leaked
}

/// The leader's actions on one member, as menu items.
fn member_actions(m: &TeamMember, is_self: bool) -> Vec<MenuItem> {
    let mut items: Vec<MenuItem> = Role::ALL
        .iter()
        .filter(|r| !(m.roster_type == "starter" && Role::parse(&m.role) == Some(**r)))
        .map(|r| MenuItem::new(r.key(), format!("Start as {}", r.label())))
        .collect();
    if m.roster_type != "substitute" {
        items.push(MenuItem::new("substitute", "Move to the bench"));
    }
    if m.roster_type != "coach" {
        items.push(MenuItem::new("coach", "Make coach"));
    }
    if m.roster_type != "member" {
        items.push(MenuItem::new("member", "Take off the roster"));
    }
    if !is_self {
        items.push(MenuItem::new("lead", "Make team leader"));
        items.push(MenuItem::new("kick", "Remove from team").danger());
    }
    items
}

/// The team page's resources (see `TeamDashboard`).
#[derive(Clone, Copy)]
struct TeamResources {
    requests: Resource<Result<Vec<JoinRequest>, ServerFnError>>,
    notes: Resource<Result<Vec<TeamNote>, ServerFnError>>,
    games: Resource<Result<Vec<RecentMatch>, ServerFnError>>,
    gaps: Resource<Result<Vec<PoolGapWarning>, ServerFnError>>,
}

#[component]
pub fn TeamDashboard() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });
    let team = Resource::new(|| (), |_| get_team_dashboard());
    // Every resource of the page is created here, before any Suspense, so
    // the server and the browser register them in the same order.
    let res = TeamResources {
        requests: Resource::new(|| (), |_| get_pending_requests()),
        notes: Resource::new(|| (), |_| get_all_team_notes()),
        games: Resource::new(|| (), |_| get_recent_team_matches()),
        gaps: Resource::new(|| (), |_| get_pool_gap_panel()),
    };
    let reload = Callback::new(move |_| {
        team.refetch();
        res.requests.refetch();
    });

    view! {
        <PageShell attr:data-feedback-label="Team">
            <Suspense fallback=|| view! {
                <SkeletonGroup label="Loading the team">
                    <Skeleton width="w-64 max-w-full" size="h-10" />
                    <Skeleton shape=SkeletonShape::Block size="h-64" />
                </SkeletonGroup>
            }>
                {move || Suspend::new(async move {
                    match team.await {
                        Ok(Some((t, members, me))) => view! { <TeamView team=t members=members me=me reload=reload res=res /> }.into_any(),
                        Ok(None) => view! {
                            <PageHeader title="Team" />
                            <NoTeamState />
                        }.into_any(),
                        Err(_) => view! {
                            <PageHeader title="Team" />
                            <EmptyState title="The team did not load" message="The server did not answer. Reload the page in a moment." />
                        }.into_any(),
                    }
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn TeamView(
    team: Team,
    members: Vec<TeamMember>,
    me: String,
    reload: Callback<()>,
    res: TeamResources,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let is_leader = team.created_by == me;
    let leader_name = members
        .iter()
        .find(|m| m.user_id == team.created_by)
        .map(|m| m.username.clone())
        .unwrap_or_default();
    let groups = group_roster(&members);
    let edit_open = RwSignal::new(false);
    let leave_open = RwSignal::new(false);
    let kick_target = RwSignal::new(None::<(String, String)>);
    let kick_open = RwSignal::new(false);
    let lead_open = RwSignal::new(false);
    let member_count = members.len();
    let region = team.region.clone();

    let on_action = Callback::new(move |(user_id, name, action): (String, String, String)| {
        if action == "kick" {
            kick_target.set(Some((user_id, name)));
            kick_open.set(true);
            return;
        }
        if action == "lead" {
            kick_target.set(Some((user_id, name)));
            lead_open.set(true);
            return;
        }
        leptos::task::spawn_local(async move {
            match set_member_place(user_id, action.clone()).await {
                Ok(()) => {
                    let what = match action.as_str() {
                        "substitute" => format!("{name} is on the bench"),
                        "coach" => format!("{name} is a coach"),
                        "member" => format!("{name} is off the roster"),
                        pos => format!(
                            "{name} starts as {}",
                            Role::parse(pos).map(|r| r.label()).unwrap_or(pos)
                        ),
                    };
                    toast.push.run(Toast::success(what));
                    reload.run(());
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let kick = Callback::new(move |_| {
        let Some((id, name)) = kick_target.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match kick_member(id).await {
                Ok(()) => {
                    toast
                        .push
                        .run(Toast::success(format!("{name} was removed from the team")));
                    reload.run(());
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let hand_over = Callback::new(move |_| {
        let Some((id, name)) = kick_target.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match hand_over_leadership(id).await {
                Ok(()) => {
                    toast
                        .push
                        .run(Toast::success(format!("{name} now leads the team")));
                    reload.run(());
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(crate::models::utils::error_text(&e))),
            }
        });
    });
    let leave = Callback::new(move |_| {
        leptos::task::spawn_local(async move {
            match leave_team().await {
                Ok(()) =>
                {
                    #[cfg(feature = "hydrate")]
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().set_href("/team/roster");
                    }
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });

    let me_row = StoredValue::new(me.clone());
    let row = move |m: Option<TeamMember>, position: Option<Role>| -> AnyView {
        match m {
            None => view! {
                <li class="grid grid-cols-[3rem_minmax(0,1fr)] items-center gap-3 border-t border-rule py-2.5" data-roster-row="open">
                    <span>{position.map(|r| view! { <RoleLabel role=r /> })}</span>
                    <span class="text-body text-muted">"Open: no starter in this position"</span>
                </li>
            }
            .into_any(),
            Some(m) => {
                let is_self = m.user_id == me_row.get_value();
                let items = member_actions(&m, is_self);
                let uid = m.user_id.clone();
                let uname = m.username.clone();
                let card_href = format!("/team/player/{}", m.user_id);
                let shown_role = position.or_else(|| Role::parse(&m.role));
                let menu_id = intern(format!("member-{}", m.user_id.replace(':', "-")));
                view! {
                    <li class="grid grid-cols-[3rem_minmax(0,1fr)_auto] items-center gap-3 border-t border-rule py-2" data-roster-row=m.user_id.clone() data-place=m.roster_type.clone()>
                        <span>{shown_role.map(|r| view! { <RoleLabel role=r /> })}</span>
                        <span class="flex min-w-0 items-center gap-2.5">
                            <UserAvatar name=m.username.clone() size=AvatarSize::Sm />
                            <span class="min-w-0">
                                <A href=card_href attr:class="block truncate font-medium text-primary underline decoration-transparent underline-offset-4 hover:decoration-outline">{m.username.clone()}</A>
                                <span class="block truncate text-caption text-muted">
                                    {m.riot_summoner_name.clone().unwrap_or_else(|| "No Riot account linked".to_string())}
                                    {is_self.then_some(" \u{00b7} you")}
                                </span>
                            </span>
                        </span>
                        {is_leader.then(|| view! {
                            <Menu
                                id=menu_id
                                label=format!("Roster actions for {uname}")
                                items=items
                                align=MenuAlign::End
                                variant=ButtonVariant::Quiet
                                size=ButtonSize::Sm
                                trigger=move || view! {
                                    <span>"Actions"</span>
                                    <svg class="size-3.5 text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
                                    </svg>
                                }
                                on_select=Callback::new(move |a: String| on_action.run((uid.clone(), uname.clone(), a)))
                            />
                        })}
                    </li>
                }
                .into_any()
            }
        }
    };

    let starters_view = groups
        .starters
        .clone()
        .into_iter()
        .zip(Role::ALL)
        .map(|(m, r)| row(m, Some(r)))
        .collect_view();
    let group_list = move |title: &'static str,
                           id: &'static str,
                           list: Vec<TeamMember>,
                           empty: &'static str| {
        view! {
            <div class="mt-8" data-roster-group=id>
                <h3 id=format!("roster-{id}") class="mb-2 flex items-baseline justify-between font-display text-heading font-semibold text-primary">
                    {title}
                    <span class="figures text-label font-normal text-muted">{list.len()}</span>
                </h3>
                {if list.is_empty() {
                    view! { <p class="border-t border-rule py-2.5 text-body text-muted">{empty}</p> }.into_any()
                } else {
                    view! {
                        <ul aria-labelledby=format!("roster-{id}") class="border-b border-rule">
                            {list.into_iter().map(|m| row(Some(m), None)).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </div>
        }
    };
    let unplaced = groups.unplaced.clone();

    view! {
        <PageHeader
            title=team.name.clone()
            eyebrow="Team"
            meta=move || view! {
                <span class="figures">{region.clone()}</span>
                <span aria-hidden="true" class="text-muted">"/"</span>
                <span>{if member_count == 1 { "1 member".to_string() } else { format!("{member_count} members") }}</span>
                <span aria-hidden="true" class="text-muted">"/"</span>
                <span>{if is_leader { "You lead this team".to_string() } else { format!("Led by {leader_name}") }}</span>
            }
            actions=move || view! {
                <Button href="/team/roster">"Roster and joining"</Button>
                {if is_leader {
                    view! { <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| edit_open.set(true))>"Edit team"</Button> }.into_any()
                } else {
                    view! { <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| leave_open.set(true))>"Leave team"</Button> }.into_any()
                }}
            }
        />
        <div class="grid gap-x-12 gap-y-12 lg:grid-cols-[minmax(0,1fr)_minmax(0,26rem)]">
            <div class="flex min-w-0 flex-col gap-12">
                <section aria-labelledby="roster-h" data-roster>
                    <SectionHeader title="Roster" id="roster-h" meta=if is_leader { "Use a player's actions to start, bench or make them coach." } else { "The leader places players." } />
                    <h3 id="roster-starters" class="mb-2 font-display text-heading font-semibold text-primary">"Starting five"</h3>
                    <ul aria-labelledby="roster-starters" class="border-b border-rule" data-roster-group="starters">{starters_view}</ul>
                    {group_list("Substitutes", "substitutes", groups.substitutes.clone(), "No one on the bench. Substitutes play when needed and see everything the team does.")}
                    {group_list("Coaches", "coaches", groups.coaches.clone(), "No coach on the roster yet.")}
                    {(!unplaced.is_empty()).then(|| group_list("Not placed yet", "unplaced", unplaced.clone(), ""))}
                </section>
                <RecentGames games=res.games />
                <PoolGaps gaps=res.gaps />
            </div>
            <div class="flex min-w-0 flex-col gap-12">
                {is_leader.then(|| view! { <JoinRequests reload=reload requests=res.requests /> })}
                <TeamNotes notes=res.notes />
            </div>
        </div>
        {is_leader.then(|| view! { <EditTeam open=edit_open name=team.name.clone() team_region=team.region.clone() reload=reload /> })}
        <ConfirmDialog
            id="leave-team"
            open=leave_open
            title="Leave the team?"
            message="You lose access to the team's drafts, plans, reviews and notes. Your own pages stay. To come back, ask the leader again."
            confirm_label="Leave team"
            tone=ConfirmTone::Destructive
            on_confirm=leave
        />
        <ConfirmDialog
            id="hand-over-leadership"
            open=lead_open
            title="Hand over leadership?"
            message="The new leader places players, answers join requests and edits the team. You stay on the team as a member."
            confirm_label="Make team leader"
            on_confirm=hand_over
        />
        <ConfirmDialog
            id="kick-member"
            open=kick_open
            title="Remove this player?"
            message="They lose access to the team's drafts, plans, reviews and notes. They can ask to join again."
            confirm_label="Remove from team"
            tone=ConfirmTone::Destructive
            on_confirm=kick
        />
    }
}

#[component]
fn EditTeam(
    open: RwSignal<bool>,
    name: String,
    team_region: String,
    reload: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let name = RwSignal::new(name);
    let region = RwSignal::new(team_region);
    let error = RwSignal::new(None::<String>);
    let save = Callback::new(move |_| {
        leptos::task::spawn_local(async move {
            match update_team_info(name.get_untracked(), region.get_untracked()).await {
                Ok(()) => {
                    open.set(false);
                    toast.push.run(Toast::success("Team saved"));
                    reload.run(());
                }
                Err(e) => error.set(Some(e.to_string())),
            }
        });
    });
    let options: Vec<SelectOption> = TEAM_REGIONS
        .iter()
        .map(|r| SelectOption::new(*r, *r))
        .collect();
    let options = StoredValue::new(options);
    view! {
        <Dialog
            id="edit-team"
            open=open
            title="Edit team"
            footer=move || view! {
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary on_click=save>"Save team"</Button>
            }
        >
            <div class="flex flex-col gap-4">
                <Field id="team-name" label="Team name" required=true error=Signal::derive(move || error.get())>
                    <TextField value=name maxlength=60 required=true />
                </Field>
                <Field id="team-region" label="Region" hint="Riot calls for team syncs and scouting use it.">
                    <Select value=region options=options.get_value() />
                </Field>
            </div>
        </Dialog>
    }
}

#[component]
fn JoinRequests(
    reload: Callback<()>,
    requests: Resource<Result<Vec<JoinRequest>, ServerFnError>>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let answer = Callback::new(move |(id, name, accept): (String, String, bool)| {
        leptos::task::spawn_local(async move {
            match handle_join_request(id, accept).await {
                Ok(()) => {
                    toast.push.run(Toast::success(if accept {
                        format!("{name} joined the team")
                    } else {
                        format!("Declined {name}'s request")
                    }));
                    requests.refetch();
                    reload.run(());
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    view! {
        <section aria-labelledby="requests-h" data-join-requests>
            <SectionHeader title="Join requests" id="requests-h" />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading requests"><Skeleton lines=2 /></SkeletonGroup> }>
                {move || requests.get().map(|r| {
                    let list = r.unwrap_or_default();
                    if list.is_empty() {
                        view! { <p class="text-body text-secondary">"No requests waiting. Players ask to join from the roster page; their requests appear here."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="border-b border-rule">
                                {list.into_iter().map(|req| {
                                    let (a_id, d_id) = (req.id.clone(), req.id.clone());
                                    let (a_n, d_n) = (req.username.clone(), req.username.clone());
                                    view! {
                                        <li class="flex flex-wrap items-center justify-between gap-3 border-t border-rule py-2.5">
                                            <span class="min-w-0">
                                                <span class="block font-medium text-primary">{req.username.clone()}</span>
                                                <span class="block text-caption text-muted">{req.riot_summoner_name.clone().unwrap_or_else(|| "No Riot account linked".to_string())}</span>
                                            </span>
                                            <span class="flex gap-2">
                                                <Button size=ButtonSize::Sm on_click=Callback::new(move |_| answer.run((a_id.clone(), a_n.clone(), true)))>"Accept"</Button>
                                                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| answer.run((d_id.clone(), d_n.clone(), false)))>"Decline"</Button>
                                            </span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </section>
    }
}

/// Team notes: open to every member, edits and deletes undoable from the
/// toast (the server keeps the replaced text and soft-deletes for a day).
#[component]
fn TeamNotes(notes: Resource<Result<Vec<TeamNote>, ServerFnError>>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let draft = RwSignal::new(String::new());
    let editing = RwSignal::new(None::<String>);
    let edit_text = RwSignal::new(String::new());
    let adding = RwSignal::new(false);

    let add = Callback::new(move |_| {
        let content = draft.get_untracked();
        if content.trim().is_empty() {
            return;
        }
        adding.set(true);
        leptos::task::spawn_local(async move {
            match add_team_note(content).await {
                Ok(_) => {
                    draft.set(String::new());
                    notes.refetch();
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
            adding.set(false);
        });
    });
    let save_edit = Callback::new(move |id: String| {
        let content = edit_text.get_untracked();
        leptos::task::spawn_local(async move {
            match edit_team_note(id.clone(), content).await {
                Ok(()) => {
                    editing.set(None);
                    notes.refetch();
                    let undo_id = id.clone();
                    toast.push.run(Toast::success("Note saved").with_action(
                        "Undo",
                        Callback::new(move |_| {
                            let id = undo_id.clone();
                            leptos::task::spawn_local(async move {
                                match undo_note_edit(id).await {
                                    Ok(()) => notes.refetch(),
                                    Err(e) => toast.push.run(Toast::error(e.to_string())),
                                }
                            });
                        }),
                    ));
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let delete = Callback::new(move |id: String| {
        leptos::task::spawn_local(async move {
            match remove_team_note(id.clone()).await {
                Ok(()) => {
                    notes.refetch();
                    let undo_id = id.clone();
                    toast.push.run(Toast::success("Note deleted").with_action(
                        "Undo",
                        Callback::new(move |_| {
                            let id = undo_id.clone();
                            leptos::task::spawn_local(async move {
                                match restore_note(id).await {
                                    Ok(()) => notes.refetch(),
                                    Err(e) => toast.push.run(Toast::error(e.to_string())),
                                }
                            });
                        }),
                    ));
                }
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });
    let pin = Callback::new(move |(id, pinned): (String, bool)| {
        leptos::task::spawn_local(async move {
            match toggle_note_pin(id, pinned).await {
                Ok(()) => notes.refetch(),
                Err(e) => toast.push.run(Toast::error(e.to_string())),
            }
        });
    });

    view! {
        <section aria-labelledby="notes-h" data-team-notes>
            <SectionHeader title="Team notes" id="notes-h" meta="Everyone on the team can edit them." />
            <div class="flex flex-col gap-2">
                <Field id="new-team-note" label="New note">
                    <TextField value=draft rows=3 maxlength=4000 placeholder="Something the whole team should know" />
                </Field>
                <div>
                    <Button on_click=add loading=adding attr:data-testid="add-team-note">"Add note"</Button>
                </div>
            </div>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading notes"><Skeleton lines=3 /></SkeletonGroup> }>
                {move || notes.get().map(|r| {
                    let list = r.unwrap_or_default();
                    if list.is_empty() {
                        view! { <div class="mt-6"><EmptyState size=EmptySize::Compact message="No notes yet. Notes you add appear here for the whole team, pinned ones first." /></div> }.into_any()
                    } else {
                        view! {
                            <ul class="mt-6 border-b border-rule">
                                {list.into_iter().map(|n| {
                                    let id = n.id.clone().unwrap_or_default();
                                    let (id_edit, id_save, id_del, id_pin, id_cmp, id_cmp2) = (id.clone(), id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
                                    let content = n.content.clone();
                                    let content_for_edit = n.content.clone();
                                    let pinned = n.pinned;
                                    view! {
                                        <li class="border-t border-rule py-3" data-team-note=id.clone()>
                                            <p class="mb-1 flex flex-wrap items-baseline gap-x-2 text-caption text-muted">
                                                {pinned.then(|| view! { <span class="eyebrow text-secondary">"Pinned"</span> })}
                                                <span class="font-medium text-secondary">{n.author_name.clone()}</span>
                                                {n.created_at.clone().map(|c| view! { <LocalTime iso=c style=TimeStyle::DayMonth /> })}
                                                {n.edited_by_name.clone().map(|by| view! { <span>{format!("edited by {by}")}</span> })}
                                            </p>
                                            {move || if editing.get().as_deref() == Some(id_cmp.as_str()) {
                                                let id_save = id_save.clone();
                                                view! {
                                                    <div class="flex flex-col gap-2">
                                                        <TextField value=edit_text rows=4 maxlength=4000 aria_label="Edit note" />
                                                        <div class="flex gap-2">
                                                            <Button size=ButtonSize::Sm on_click=Callback::new(move |_| save_edit.run(id_save.clone()))>"Save note"</Button>
                                                            <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| editing.set(None))>"Cancel"</Button>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <p class="whitespace-pre-line text-body text-primary" data-note-text>{content.clone()}</p> }.into_any()
                                            }}
                                            <div class="mt-1 flex gap-1" class:hidden=move || editing.get().as_deref() == Some(id_cmp2.as_str())>
                                                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| { edit_text.set(content_for_edit.clone()); editing.set(Some(id_edit.clone())); })>"Edit"</Button>
                                                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| pin.run((id_pin.clone(), !pinned)))>
                                                    {if pinned { "Unpin" } else { "Pin" }}
                                                </Button>
                                                <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet on_click=Callback::new(move |_| delete.run(id_del.clone()))>"Delete"</Button>
                                            </div>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn RecentGames(games: Resource<Result<Vec<RecentMatch>, ServerFnError>>) -> impl IntoView {
    view! {
        <section aria-labelledby="games-h" data-recent-games>
            <SectionHeader
                title="Recent games"
                id="games-h"
                actions=move || view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href="/stats">"All team games"</Button> }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading games"><Skeleton lines=3 /></SkeletonGroup> }>
                {move || games.get().map(|r| {
                    let list = r.unwrap_or_default();
                    if list.is_empty() {
                        view! { <EmptyState size=EmptySize::Compact message="No games stored yet. Members who link their Riot account can sync their games on Team games; the latest appear here." /> }.into_any()
                    } else {
                        view! {
                            <ul class="border-b border-rule">
                                {list.into_iter().map(|g| {
                                    let href = format!("/match/{}", g.riot_match_id);
                                    view! {
                                        <li class="grid grid-cols-[1.5rem_1.5rem_minmax(0,1fr)_auto] items-center gap-3 border-t border-rule py-2">
                                            <ResultGlyph result=GameResult::from_win(g.win) size=GlyphSize::Sm />
                                            <ChampionAvatar name=g.champion.clone() size=AvatarSize::Sm decorative=true />
                                            <A href=href attr:class="min-w-0 truncate text-body text-primary underline decoration-transparent underline-offset-4 hover:decoration-outline">
                                                {format!("{} on {}", g.username, g.champion)}
                                            </A>
                                            <span class="figures text-label text-muted">
                                                {format!("{}/{}/{}", g.kills, g.deaths, g.assists)}
                                                {g.game_end.clone().map(|d| view! { " \u{00b7} " <LocalTime iso=d style=TimeStyle::DayMonth /> })}
                                            </span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn PoolGaps(gaps: Resource<Result<Vec<PoolGapWarning>, ServerFnError>>) -> impl IntoView {
    view! {
        <section aria-labelledby="gaps-h" data-pool-gaps>
            <SectionHeader
                title="Pool gaps"
                id="gaps-h"
                actions=move || view! { <Button size=ButtonSize::Sm variant=ButtonVariant::Quiet href="/champion-pool">"Champion pools"</Button> }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading pool gaps"><Skeleton lines=2 /></SkeletonGroup> }>
                {move || gaps.get().map(|r| {
                    let list = r.unwrap_or_default();
                    if list.is_empty() {
                        view! { <p class="text-body text-secondary">"No gaps found. A warning appears when a player's pool for a position leans on one champion class, or lacks a class the scouted opponents play."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="border-b border-rule">
                                {list.into_iter().map(|w| {
                                    let role = Role::parse(&w.role);
                                    let mut parts = Vec::new();
                                    if let Some(c) = &w.dominant_class {
                                        parts.push(format!("mostly {c}"));
                                    }
                                    if !w.missing_classes.is_empty() {
                                        parts.push(format!("no {}, which scouted opponents play", w.missing_classes.join(", ")));
                                    }
                                    view! {
                                        <li class="grid grid-cols-[3rem_minmax(0,1fr)] items-baseline gap-3 border-t border-rule py-2">
                                            <span>{role.map(|r| view! { <RoleLabel role=r /> })}</span>
                                            <span class="text-body text-primary">
                                                <span class="font-medium">{w.username.clone()}</span>
                                                <span class="text-secondary">{format!(": {}", parts.join("; "))}</span>
                                                {w.opponent_escalated.then(|| view! { <span class="ml-2 text-label text-warning"><span aria-hidden="true">"! "</span>"Matters against your opponents"</span> })}
                                            </span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                })}
            </Suspense>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(name: &str, role: &str, place: &str) -> TeamMember {
        TeamMember {
            user_id: format!("user:{name}"),
            username: name.into(),
            role: role.into(),
            roster_type: place.into(),
            riot_summoner_name: None,
        }
    }

    #[test]
    fn roster_groups_by_place() {
        let g = group_roster(&[
            m("a", "mid", "starter"),
            m("b", "mid", "starter"),
            m("c", "jungle", "substitute"),
            m("d", "unassigned", "coach"),
            m("e", "unassigned", "member"),
            m("f", "unassigned", "starter"),
        ]);
        assert_eq!(
            g.starters[2].as_ref().map(|x| x.username.as_str()),
            Some("a")
        );
        assert!(g.starters[0].is_none());
        // A second starter in one position shows on the bench, not lost.
        assert_eq!(
            g.substitutes
                .iter()
                .map(|x| x.username.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "c"]
        );
        assert_eq!(g.coaches.len(), 1);
        assert_eq!(
            g.unplaced
                .iter()
                .map(|x| x.username.as_str())
                .collect::<Vec<_>>(),
            vec!["e", "f"]
        );
    }

    #[test]
    fn leader_actions_skip_the_current_place() {
        let items = member_actions(&m("a", "mid", "starter"), false);
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(!ids.contains(&"mid"));
        assert!(ids.contains(&"top") && ids.contains(&"substitute") && ids.contains(&"coach"));
        assert_eq!(ids.last(), Some(&"kick"));
        let own = member_actions(&m("a", "unassigned", "member"), true);
        assert!(!own.iter().any(|i| i.id == "kick" || i.id == "member"));
    }
}
