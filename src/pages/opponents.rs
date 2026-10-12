//! Opponents: the team's scouting dossiers. The list shows each opponent
//! with its evidence (players, champions seen) and when it was last
//! surveyed; the detail holds the players by role, scouting notes and the
//! Riot intel refresh. Images come only from `/img/...`.

use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, Callout, CalloutTone, ChampionAvatar,
    ConfirmDialog, ConfirmTone, Density, Dialog, EmptySize, EmptyState, Field, HeadingLevel,
    NoTeamState, PageHeader, PageShell, Role, RoleLabel, SectionHeader, Skeleton, SkeletonGroup,
    SkeletonShape, TextField, Toast, ToastContext,
};
use crate::models::opponent::{is_stale, Opponent, OpponentPlayer};
use crate::models::utils::format_timestamp;
use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_opponents() -> Result<Vec<Opponent>, ServerFnError> {
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

    db::list_opponents(&surreal, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn create_opponent(name: String) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db: surreal, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    db::create_opponent(&surreal, &team_id, name, None)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_opponent_detail(
    id: String,
) -> Result<Option<(Opponent, Vec<OpponentPlayer>)>, ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(None);
    };
    db::get_opponent(&ctx.db, &team_id, &id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_opponent(id: String, name: String, notes: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let notes_opt = if notes.is_empty() { None } else { Some(notes) };
    db::update_opponent(&ctx.db, &team_id, &id, name, notes_opt)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_opponent_action(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::delete_opponent(&ctx.db, &team_id, &id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn add_player(
    opponent_id: String,
    name: String,
    role: String,
) -> Result<String, ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::add_opponent_player(&ctx.db, &team_id, &opponent_id, name, role)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_player(
    id: String,
    name: String,
    role: String,
    riot_summoner_name: String,
    notes: String,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;

    let summoner_opt = if riot_summoner_name.is_empty() {
        None
    } else {
        Some(riot_summoner_name)
    };
    let notes_opt = if notes.is_empty() { None } else { Some(notes) };

    db::save_opponent_player_info(&ctx.db, &team_id, &id, name, role, summoner_opt, notes_opt)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn remove_player(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::delete_opponent_player(&ctx.db, &team_id, &id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn fetch_champions(
    player_id: String,
    summoner_name: String,
) -> Result<Vec<String>, ServerFnError> {
    use crate::server::{authz, db, rate_limit::Bucket, riot};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    // Check ownership before spending Riot API quota.
    db::ensure_team_opponent_player(&ctx.db, &team_id, &player_id)
        .await
        .map_err(authz::server_err)?;

    let riot_client = authz::riot()?;
    if !riot_client.is_configured() {
        return Err(ServerFnError::new("No Riot API key configured"));
    }

    // Parse summoner name: "Name#Tag" format
    let parts: Vec<&str> = summoner_name.split('#').collect();
    if parts.len() != 2 {
        return Err(ServerFnError::new(
            "Summoner name must be in Name#Tag format",
        ));
    }
    authz::rate_limit(Bucket::Riot).await?;
    // Account lookup + match list + 20 matches.
    riot_client
        .ensure_headroom(2 + 20)
        .map_err(authz::server_err)?;

    // Opponents are looked up on the team's server.
    let team_region = db::get_team_region(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let platform = riot::platform_for(&[team_region.as_deref()]);
    let puuid = riot_client
        .get_puuid(parts[0], parts[1], platform)
        .await
        .map_err(authz::server_err)?;

    let champions = riot_client
        .fetch_player_champions(&puuid, 20, platform)
        .await
        .map_err(authz::server_err)?;

    // Update the player record with champions
    db::update_opponent_player_champions(&ctx.db, &team_id, &player_id, champions.clone())
        .await
        .map_err(authz::server_err)?;

    Ok(champions)
}

/// Create an opponent with an initial set of players in a single server call.
///
/// Returns (opponent_id, player_ids) so the UI can immediately trigger per-player intel fetches
/// without an extra round-trip to get the newly created player IDs.
///
/// `players_json`: JSON-encoded `Vec<(String, Option<String>)>` where each tuple is
/// `(role, riot_summoner_name_or_none)`.
#[server]
pub async fn create_opponent_with_players_fn(
    name: String,
    players_json: String,
) -> Result<(String, Vec<String>), ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;

    let players: Vec<(String, Option<String>)> =
        serde_json::from_str(&players_json).map_err(authz::server_err)?;

    let (opponent_id, player_ids) =
        db::create_opponent_with_players(&ctx.db, &team_id, name, None, players)
            .await
            .map_err(authz::server_err)?;

    Ok((opponent_id, player_ids))
}

/// Fetch combined Riot API intel for a player (champions, per-match roles, mastery) and persist it.
///
/// Computes role distribution from per-match `team_position` data and stores as JSON.
/// Returns `Ok(())` immediately if `riot_summoner_name` is empty — no error for optional field.
#[server]
pub async fn fetch_player_intel_fn(
    player_id: String,
    riot_summoner_name: String,
) -> Result<(), ServerFnError> {
    use crate::server::{authz, db, rate_limit::Bucket, riot};

    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    // Check ownership before spending Riot API quota.
    db::ensure_team_opponent_player(&ctx.db, &team_id, &player_id)
        .await
        .map_err(authz::server_err)?;

    // Empty summoner name is not an error — skip gracefully
    if riot_summoner_name.trim().is_empty() {
        return Ok(());
    }

    let riot_client = authz::riot()?;
    if !riot_client.is_configured() {
        return Err(ServerFnError::new("No Riot API key configured"));
    }

    // Parse "GameName#TagLine" format
    let parts: Vec<&str> = riot_summoner_name.split('#').collect();
    if parts.len() != 2 {
        return Err(ServerFnError::new(
            "Summoner name must be in Name#Tag format",
        ));
    }
    authz::rate_limit(Bucket::Riot).await?;
    // Account lookup + match list + 20 matches + masteries.
    riot_client
        .ensure_headroom(3 + 20)
        .map_err(authz::server_err)?;

    // Opponents are looked up on the team's server.
    let team_region = db::get_team_region(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let platform = riot::platform_for(&[team_region.as_deref()]);
    let puuid = riot_client
        .get_puuid(parts[0], parts[1], platform)
        .await
        .map_err(authz::server_err)?;

    let intel = riot_client
        .fetch_player_intel(&puuid, 20, platform)
        .await
        .map_err(authz::server_err)?;

    // Compute role distribution: count occurrences of each team_position
    let mut role_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for (_champion, position) in &intel.champion_with_role {
        if !position.is_empty() {
            *role_counts.entry(position.clone()).or_insert(0) += 1;
        }
    }
    let role_dist: Vec<(String, u32)> = role_counts.into_iter().collect();

    let mastery_json = serde_json::to_string(&intel.mastery_data).unwrap_or_default();
    let role_dist_json = serde_json::to_string(&role_dist).unwrap_or_default();

    db::update_opponent_player_intel(
        &ctx.db,
        &team_id,
        &player_id,
        Some(puuid),
        intel.recent_champions,
        Some(mastery_json),
        Some(role_dist_json),
    )
    .await
    .map_err(authz::server_err)?;

    Ok(())
}

/// One row of the dossier list: the opponent and the evidence behind it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OpponentDossier {
    pub opponent: Opponent,
    pub players: usize,
    /// Games in the scouted match history, summed over players.
    /// Champions seen per player, summed. Riot refreshes store each
    /// player's distinct champions, not one entry per game, so this is
    /// never a game count.
    pub champions_seen: usize,
    /// "3 days ago" (computed on the server), `None` if never surveyed.
    pub surveyed: Option<String>,
    /// Surveyed a week or more ago.
    pub stale: bool,
}

/// The team's opponents with player counts, champions seen and the date of
/// the latest survey.
#[server]
pub async fn get_opponent_dossiers() -> Result<Vec<OpponentDossier>, ServerFnError> {
    use crate::server::{authz, db};

    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    let list = db::list_opponents(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)?;
    let mut out = Vec::with_capacity(list.len());
    for opponent in list {
        let id = opponent.id.clone().unwrap_or_default();
        let players = db::get_opponent(&ctx.db, &team_id, &id)
            .await
            .map_err(authz::server_err)?
            .map(|(_, p)| p)
            .unwrap_or_default();
        let latest = players
            .iter()
            .filter_map(|p| p.last_fetched.clone())
            .filter(|s| !s.is_empty())
            .max();
        out.push(OpponentDossier {
            players: players
                .iter()
                .filter(|p| !p.name.is_empty() || p.riot_summoner_name.is_some())
                .count(),
            champions_seen: players.iter().map(|p| p.recent_champions.len()).sum(),
            surveyed: latest.as_deref().map(format_timestamp),
            stale: latest.as_deref().map(is_stale).unwrap_or(false),
            opponent,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Roles in roster order, as stored on opponent players.
const ROLES: [&str; 5] = ["top", "jungle", "mid", "bot", "support"];

fn valid_riot_id(s: &str) -> bool {
    let s = s.trim();
    s.is_empty()
        || matches!(s.split_once('#'), Some((a, b)) if !a.trim().is_empty() && !b.trim().is_empty())
}

/// A scouted player's name: the name, else the Riot ID, else a placeholder.
fn player_name(p: &OpponentPlayer) -> String {
    if !p.name.trim().is_empty() {
        return p.name.clone();
    }
    p.riot_summoner_name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "Unnamed player".to_string())
}

fn position_label(pos: &str) -> &str {
    match pos {
        "TOP" => "Top",
        "JUNGLE" => "Jungle",
        "MIDDLE" => "Mid",
        "BOTTOM" => "Bot",
        "UTILITY" => "Support",
        other => other,
    }
}

const RIOT_BUDGET_NOTE: &str = "Refreshing reads the player's last 20 games and champion mastery from Riot: about 23 requests per player. The server shares one request budget across the app, so a refresh can be refused for a minute when it is used up.";

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

/// "5 players, 12 champions seen" for a dossier row.
fn evidence_line(players: usize, champions: usize) -> String {
    let p = match players {
        0 => return "No players yet".to_string(),
        1 => "1 player".to_string(),
        n => format!("{n} players"),
    };
    match champions {
        0 => format!("{p}, no champions seen"),
        1 => format!("{p}, 1 champion seen"),
        c => format!("{p}, {c} champions seen"),
    }
}

/// "surveyed 3 days ago" (flagged when a week or older) or "never surveyed".
fn surveyed_line(surveyed: Option<&str>, stale: bool) -> String {
    match surveyed {
        Some(s) if stale => format!("surveyed {s}, over a week"),
        Some(s) => format!("surveyed {s}"),
        None => "never surveyed".to_string(),
    }
}

#[component]
pub fn OpponentsPage() -> impl IntoView {
    let user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || match user.get() {
        Some(Ok(None)) =>
        {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
        _ => {}
    });

    let params = leptos_router::hooks::use_query_map();
    let dossiers = Resource::new(|| (), |_| get_opponent_dossiers());
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
    let selected = RwSignal::new(None::<String>);
    // `?opponent_id=` (from Match Day's brief) opens that dossier.
    Effect::new(move |_| {
        if let Some(id) = params.read().get("opponent_id").filter(|i| !i.is_empty()) {
            if selected.get_untracked().is_none() {
                selected.set(Some(id));
            }
        }
    });
    let detail = Resource::new(
        move || selected.get(),
        |id| async move {
            match id {
                Some(id) => get_opponent_detail(id).await,
                None => Ok(None),
            }
        },
    );
    let creating = RwSignal::new(false);

    let back_to_draft = move || {
        (params.read().get("return_to").as_deref() == Some("draft")).then(|| {
            match params.read().get("draft_id").filter(|d| !d.is_empty()) {
                Some(d) => format!("/draft?draft_id={d}"),
                None => "/draft".to_string(),
            }
        })
    };

    let on_deleted = Callback::new(move |_: ()| {
        selected.set(None);
        dossiers.refetch();
    });
    let on_changed = Callback::new(move |_: ()| {
        detail.refetch();
        dossiers.refetch();
    });
    let on_created = Callback::new(move |id: String| {
        dossiers.refetch();
        selected.set(Some(id));
    });

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Opponents">
            <PageHeader
                title="Opponents"
                eyebrow="Scouting"
                meta=move || view! { <span>"A dossier per team you play: the players, what they play, and your notes."</span> }
                actions=move || view! {
                    {move || back_to_draft().map(|href| view! { <Button variant=ButtonVariant::Quiet href=href>"Back to the draft"</Button> })}
                    <Button on_click=Callback::new(move |_| creating.set(true)) attr:data-testid="new-opponent">"New opponent"</Button>
                }
            />
            {move || {
                view! {
                    <div class="grid grid-cols-1 gap-x-10 gap-y-10 lg:grid-cols-[18rem_minmax(0,1fr)]">
                        <section aria-labelledby="dossiers-heading" class="flex min-w-0 flex-col gap-3">
                            <SectionHeader title="Dossiers" id="dossiers-heading" />
                            <Suspense fallback=|| view! { <SkeletonGroup label="Loading opponents"><Skeleton shape=SkeletonShape::Line lines=4 /></SkeletonGroup> }>
                                {move || dossiers.get().map(|r| match r {
                                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"The opponents did not load."</Callout> }.into_any(),
                                    Ok(list) if list.is_empty() => {
                                        if matches!(has_team.get(), Some(false)) {
                                            view! { <NoTeamState /> }.into_any()
                                        } else {
                                            view! {
                                                <EmptyState size=EmptySize::Compact message="No opponents yet. Add a team you will play; with their Riot IDs the dossier fills with the champions they play." />
                                            }.into_any()
                                        }
                                    }
                                    Ok(list) => view! {
                                        <ul class="flex flex-col border-t border-rule" data-testid="dossier-list">
                                            {list.into_iter().map(|d| {
                                                let id = d.opponent.id.clone().unwrap_or_default();
                                                let id_click = id.clone();
                                                let evidence = evidence_line(d.players, d.champions_seen);
                                                let surveyed = surveyed_line(d.surveyed.as_deref(), d.stale);
                                                let stale = d.stale;
                                                view! {
                                                    <li class="border-b border-rule">
                                                        <Button
                                                            variant=ButtonVariant::Quiet
                                                            size=ButtonSize::Lg
                                                            full_width=true
                                                            pressed=Signal::derive(move || selected.get().as_deref() == Some(id.as_str()))
                                                            on_click=Callback::new(move |_| { creating.set(false); selected.set(Some(id_click.clone())); })
                                                        >
                                                            <span class="flex w-full min-w-0 flex-col items-start text-left leading-tight">
                                                                <span class="w-full truncate text-label">{d.opponent.name.clone()}</span>
                                                                <span class="w-full truncate text-caption font-normal text-muted">
                                                                    {evidence}" \u{b7} "
                                                                    <span class=if stale { "text-warning" } else { "" }>{surveyed}</span>
                                                                </span>
                                                            </span>
                                                        </Button>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    }.into_any(),
                                })}
                            </Suspense>
                        </section>
                        <div class="min-w-0">
                            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the dossier"><Skeleton shape=SkeletonShape::Block size="h-64 w-full" /></SkeletonGroup> }>
                                {move || detail.get().map(|r| match r {
                                    Ok(Some((opp, players))) => view! {
                                        <OpponentDetail opponent=opp players=players on_changed=on_changed on_deleted=on_deleted />
                                    }.into_any(),
                                    Ok(None) => view! {
                                        <EmptyState
                                            title="No dossier open"
                                            level=HeadingLevel::H2
                                            message="Choose an opponent from the list, or add a new one. A dossier shows each player's role, recent champions and mastery, with the date it was last surveyed."
                                        />
                                    }.into_any(),
                                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"The dossier did not load."</Callout> }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    </div>
                }.into_any()
            }}
            <NewOpponentDialog open=creating on_created=on_created />
        </PageShell>
    }
}

/// Name and five Riot IDs; creating fetches each player's games.
#[component]
fn NewOpponentDialog(open: RwSignal<bool>, on_created: Callback<String>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let name = RwSignal::new(String::new());
    let name_error = RwSignal::new(None::<String>);
    let ids: [RwSignal<String>; 5] = std::array::from_fn(|_| RwSignal::new(String::new()));
    let errors: [RwSignal<Option<String>>; 5] = std::array::from_fn(|_| RwSignal::new(None));
    let saving = RwSignal::new(false);
    let create = Callback::new(move |_: leptos::ev::MouseEvent| {
        let n = name.get_untracked();
        if n.trim().is_empty() {
            name_error.set(Some("Name the team.".into()));
            return;
        }
        name_error.set(None);
        let mut bad = false;
        for i in 0..5 {
            let ok = valid_riot_id(&ids[i].get_untracked());
            errors[i].set((!ok).then(|| "Use Name#Tag, for example Faker#KR1.".to_string()));
            bad |= !ok;
        }
        if bad {
            return;
        }
        let riot: Vec<String> = ids
            .iter()
            .map(|s| s.get_untracked().trim().to_string())
            .collect();
        let players: Vec<(String, Option<String>)> = ROLES
            .iter()
            .zip(riot.iter())
            .map(|(r, id)| (r.to_string(), (!id.is_empty()).then(|| id.clone())))
            .collect();
        let json = serde_json::to_string(&players).unwrap_or_default();
        saving.set(true);
        leptos::task::spawn_local(async move {
            match create_opponent_with_players_fn(n.trim().to_string(), json).await {
                Ok((oid, player_ids)) => {
                    open.set(false);
                    name.set(String::new());
                    for s in ids {
                        s.set(String::new());
                    }
                    on_created.run(oid.clone());
                    let mut fetched = 0;
                    let mut failed = 0;
                    for (pid, rid) in player_ids.into_iter().zip(riot) {
                        if rid.is_empty() {
                            continue;
                        }
                        match fetch_player_intel_fn(pid, rid).await {
                            Ok(()) => fetched += 1,
                            Err(_) => failed += 1,
                        }
                    }
                    on_created.run(oid);
                    saving.set(false);
                    let msg = match (fetched, failed) {
                        (0, 0) => "Opponent added. Add Riot IDs to fill the dossier.".to_string(),
                        (f, 0) => format!("Opponent added; {f} players surveyed."),
                        (f, x) => format!("Opponent added; {f} players surveyed, {x} could not be fetched (check the Riot ID or try again later)."),
                    };
                    toast.push.run(Toast::success(msg));
                }
                Err(e) => {
                    saving.set(false);
                    toast
                        .push
                        .run(Toast::error(format!("Could not add the opponent: {e}")));
                }
            }
        });
    });
    view! {
        <Dialog
            id="new-opponent"
            open=open
            title="New opponent"
            description="Riot IDs are optional; with them the dossier fills with each player's recent champions."
            dismiss_on_scrim=false
            footer=move || view! {
                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                <Button variant=ButtonVariant::Primary on_click=create loading=Signal::derive(move || saving.get())>"Add opponent"</Button>
            }
        >
            <div class="flex flex-col gap-4 px-5 py-4">
                <Field id="opp-new-name" label="Team name" required=true error=Signal::derive(move || name_error.get())>
                    <TextField value=name placeholder="Riverside Rooks" />
                </Field>
                {ROLES.iter().enumerate().map(|(i, r)| {
                    let role = Role::parse(r).map(|x| x.label()).unwrap_or(r);
                    view! {
                        <Field id=format!("opp-new-{r}") label=format!("{role} Riot ID") error=Signal::derive(move || errors[i].get())>
                            <TextField value=ids[i] placeholder="Name#Tag" />
                        </Field>
                    }
                }).collect_view()}
            </div>
        </Dialog>
    }
}

/// One opponent's dossier: name, notes, players by role.
#[component]
fn OpponentDetail(
    opponent: Opponent,
    players: Vec<OpponentPlayer>,
    on_changed: Callback<()>,
    on_deleted: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let id = StoredValue::new(opponent.id.clone().unwrap_or_default());
    let name = RwSignal::new(opponent.name.clone());
    let notes = RwSignal::new(opponent.notes.clone().unwrap_or_default());
    let confirm = RwSignal::new(false);
    let refreshing = RwSignal::new(false);
    let players = StoredValue::new(players);

    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let (i, n, t) = (id.get_value(), name.get_untracked(), notes.get_untracked());
        if n.trim().is_empty() {
            toast.push.run(Toast::error("The team needs a name."));
            return;
        }
        leptos::task::spawn_local(async move {
            match save_opponent(i, n, t).await {
                Ok(()) => {
                    on_changed.run(());
                    toast.push.run(Toast::success("Dossier saved"));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    });
    let delete = Callback::new(move |_| {
        let i = id.get_value();
        leptos::task::spawn_local(async move {
            match delete_opponent_action(i).await {
                Ok(()) => {
                    on_deleted.run(());
                    toast.push.run(Toast::success("Opponent deleted"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });
    let refresh_all = Callback::new(move |_: leptos::ev::MouseEvent| {
        let list: Vec<(String, String)> = players.with_value(|ps| {
            ps.iter()
                .filter_map(|p| {
                    Some((
                        p.id.clone()?,
                        p.riot_summoner_name
                            .clone()
                            .filter(|r| !r.trim().is_empty())?,
                    ))
                })
                .collect()
        });
        if list.is_empty() {
            toast.push.run(Toast::info("No player has a Riot ID yet."));
            return;
        }
        refreshing.set(true);
        leptos::task::spawn_local(async move {
            let total = list.len();
            let mut ok = 0;
            let mut last_err = None;
            for (pid, rid) in list {
                match fetch_player_intel_fn(pid, rid).await {
                    Ok(()) => ok += 1,
                    Err(e) => last_err = Some(e.to_string()),
                }
            }
            refreshing.set(false);
            on_changed.run(());
            match last_err {
                None => toast
                    .push
                    .run(Toast::success(format!("{ok} of {total} players surveyed"))),
                Some(e) => toast.push.run(Toast::error(format!(
                    "{ok} of {total} players surveyed. Last error: {e}"
                ))),
            }
        });
    });

    view! {
        <article aria-labelledby="dossier-heading" class="flex min-w-0 flex-col gap-8" data-testid="dossier">
            <div class="flex flex-wrap items-baseline justify-between gap-3 border-b border-rule pb-3">
                <h2 id="dossier-heading" class="font-display text-title font-semibold text-primary">{move || name.get()}</h2>
                <div class="flex flex-wrap gap-2">
                    <Button size=ButtonSize::Sm on_click=refresh_all loading=Signal::derive(move || refreshing.get())>"Refresh from Riot"</Button>
                    <Button size=ButtonSize::Sm variant=ButtonVariant::Danger on_click=Callback::new(move |_| confirm.set(true))>"Delete"</Button>
                </div>
            </div>
            <p class="max-w-prose text-caption text-muted">{RIOT_BUDGET_NOTE}</p>
            <section aria-labelledby="dossier-notes-heading" class="flex flex-col gap-4">
                <SectionHeader title="Team" id="dossier-notes-heading" level=HeadingLevel::H3 />
                <div class="grid grid-cols-1 gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,2fr)]">
                    <Field id="opp-name" label="Name">
                        <TextField value=name />
                    </Field>
                    <Field id="opp-notes" label="Scouting notes">
                        <TextField value=notes rows=3 placeholder="How they draft, where they play, what they avoid" />
                    </Field>
                </div>
                <div><Button variant=ButtonVariant::Primary on_click=save>"Save dossier"</Button></div>
            </section>
            <section aria-labelledby="dossier-players-heading" class="flex flex-col gap-3">
                <SectionHeader title="Players" id="dossier-players-heading" level=HeadingLevel::H3 />
                <ul class="flex flex-col border-t border-rule">
                    {ROLES.iter().map(|role| {
                        let player = players.with_value(|ps| ps.iter().find(|p| Role::parse(&p.role) == Role::parse(role)).cloned());
                        match player {
                            // A slot created with the dossier but never filled.
                            Some(p) if p.name.trim().is_empty() && p.riot_summoner_name.as_deref().unwrap_or("").is_empty() => view! {
                                <EmptyRole role=role opponent_id=id.get_value() existing=p.id.clone() on_changed=on_changed />
                            }.into_any(),
                            Some(p) => view! { <PlayerRow player=p on_changed=on_changed /> }.into_any(),
                            None => view! { <EmptyRole role=role opponent_id=id.get_value() on_changed=on_changed /> }.into_any(),
                        }
                    }).collect_view()}
                </ul>
            </section>
            <ConfirmDialog
                id="delete-opponent"
                open=confirm
                title="Delete this opponent?"
                message="The dossier and its players are deleted. Drafts against them keep the name."
                confirm_label="Delete opponent"
                tone=ConfirmTone::Destructive
                on_confirm=delete
            />
        </article>
    }
}

/// A role without a player: add one by Riot ID (or a name). `existing` is
/// the empty slot record the dossier was created with, filled in place.
#[component]
fn EmptyRole(
    role: &'static str,
    opponent_id: String,
    #[prop(default = None)] existing: Option<String>,
    on_changed: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let value = RwSignal::new(String::new());
    let label = Role::parse(role).map(|r| r.label()).unwrap_or(role);
    let opp = StoredValue::new(opponent_id);
    let add = Callback::new(move |_: leptos::ev::MouseEvent| {
        let v = value.get_untracked().trim().to_string();
        if v.is_empty() {
            toast
                .push
                .run(Toast::error("Enter the player's Riot ID or name."));
            return;
        }
        let o = opp.get_value();
        let slot = existing.clone();
        leptos::task::spawn_local(async move {
            // "Name#Tag" is a Riot ID; anything else is a display name.
            let (name, riot) = if v.contains('#') {
                (String::new(), v)
            } else {
                (v, String::new())
            };
            let result = match slot {
                Some(pid) => save_player(pid, name, role.to_string(), riot, String::new()).await,
                None => match add_player(o, name.clone(), role.to_string()).await {
                    Ok(pid) if !riot.is_empty() => {
                        save_player(pid, name, role.to_string(), riot, String::new()).await
                    }
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                },
            };
            match result {
                Ok(()) => {
                    on_changed.run(());
                    toast
                        .push
                        .run(Toast::success(format!("{label} player added")));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not add: {e}"))),
            }
        });
    });
    view! {
        <li class="grid grid-cols-1 items-end gap-2 border-b border-rule py-3 sm:grid-cols-[3rem_minmax(0,1fr)_auto]" data-role=role>
            {Role::parse(role).map(|r| view! { <span class="pb-2"><RoleLabel role=r /></span> })}
            <Field id=format!("opp-add-{role}") label=format!("{label} player")>
                <TextField value=value placeholder="Name#Tag or a name" />
            </Field>
            <Button on_click=add>"Add player"</Button>
        </li>
    }
}

/// A scouted player: Riot ID, recent champions with mastery, role mix.
#[component]
fn PlayerRow(player: OpponentPlayer, on_changed: Callback<()>) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let pid = StoredValue::new(player.id.clone().unwrap_or_default());
    let riot_id = RwSignal::new(player.riot_summoner_name.clone().unwrap_or_default());
    let pname = StoredValue::new(player.name.clone());
    let prole = StoredValue::new(player.role.clone());
    let pnotes = StoredValue::new(player.notes.clone().unwrap_or_default());
    let fetching = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let confirm = RwSignal::new(false);
    let intel = player.compute_intel();
    let mastery: std::collections::HashMap<String, i32> = player
        .mastery_data()
        .into_iter()
        .map(|(c, l, _)| (c, l))
        .collect();
    let seen = player.recent_champions.len();
    let roles = player.role_distribution();
    let surveyed = player.last_fetched.clone().filter(|s| !s.is_empty());
    let stale = surveyed.as_deref().map(is_stale).unwrap_or(false);
    let display = player_name(&player);
    let role = Role::parse(&player.role);
    let has_riot_id = player
        .riot_summoner_name
        .as_deref()
        .is_some_and(|r| !r.trim().is_empty());

    let refresh = Callback::new(move |_: leptos::ev::MouseEvent| {
        let rid = riot_id.get_untracked().trim().to_string();
        if !valid_riot_id(&rid) || rid.is_empty() {
            error.set(Some("Use Name#Tag, for example Faker#KR1.".into()));
            return;
        }
        error.set(None);
        fetching.set(true);
        let id = pid.get_value();
        leptos::task::spawn_local(async move {
            // Keep the Riot ID with the player, then read their games.
            let _ = save_player(
                id.clone(),
                pname.get_value(),
                prole.get_value(),
                rid.clone(),
                pnotes.get_value(),
            )
            .await;
            match fetch_player_intel_fn(id, rid).await {
                Ok(()) => {
                    on_changed.run(());
                    toast.push.run(Toast::success("Player surveyed"));
                }
                Err(e) => error.set(Some(e.to_string())),
            }
            fetching.set(false);
        });
    });
    let remove = Callback::new(move |_| {
        let id = pid.get_value();
        leptos::task::spawn_local(async move {
            match remove_player(id).await {
                Ok(()) => {
                    on_changed.run(());
                    toast.push.run(Toast::success("Player removed"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not remove: {e}"))),
            }
        });
    });
    let field_id = format!("opp-riot-{}", player.role);
    view! {
        <li class="flex flex-col gap-3 border-b border-rule py-4" data-role=player.role.clone()>
            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                {role.map(|r| view! { <RoleLabel role=r /> })}
                <span class="font-display text-heading font-semibold text-primary">{display}</span>
                <span class=if stale { "text-caption text-warning" } else { "text-caption text-muted" }>
                    {match &surveyed {
                        Some(s) => format!("Surveyed {}{}", format_timestamp(s), if stale { " (over a week)" } else { "" }),
                        None => "Never surveyed".to_string(),
                    }}
                </span>
                <span class="figures text-caption text-muted">{match seen { 0 => "no champions seen".to_string(), 1 => "1 champion seen".to_string(), n => format!("{n} champions seen") }}</span>
            </div>
            {if intel.champion_frequencies.is_empty() {
                view! {
                    <p class="text-label text-secondary">
                        {if has_riot_id { "No games read yet. Refresh to read their last 20 games." } else { "No champions yet. Add the Riot ID and refresh to read their last 20 games." }}
                    </p>
                }.into_any()
            } else {
                view! {
                    <ul class="flex flex-wrap gap-x-4 gap-y-2" aria-label="Champions in their recent games">
                        {intel.champion_frequencies.iter().take(8).map(|(c, n)| {
                            let level = mastery.get(c).copied();
                            view! {
                                <li class="inline-flex items-center gap-1.5">
                                    <ChampionAvatar name=c.clone() size=AvatarSize::Sm decorative=true />
                                    <span class="text-label text-primary">{c.clone()}</span>
                                    <span class="figures text-caption text-muted">{format!("\u{00d7}{n}")}</span>
                                    {level.map(|l| view! { <span class="figures text-caption text-muted">{format!("M{l}")}</span> })}
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
            {(intel.otp_champion.is_some() || !roles.is_empty()).then(|| {
                let total: u32 = roles.iter().map(|(_, c)| c).sum();
                let mut mix: Vec<(String, u32)> = roles.clone();
                mix.sort_by_key(|a| std::cmp::Reverse(a.1));
                let mix_text = mix.iter().map(|(r, c)| format!("{} {}%", position_label(r), (c * 100).checked_div(total).unwrap_or(0))).collect::<Vec<_>>().join(", ");
                view! {
                    <p class="flex flex-wrap gap-x-4 text-caption text-secondary">
                        {intel.otp_champion.clone().map(|c| view! { <span class="text-warning">{format!("One-trick: {c}")}</span> })}
                        {(!mix_text.is_empty()).then(|| view! { <span>"Roles played: " {mix_text}</span> })}
                    </p>
                }
            })}
            <div class="grid grid-cols-1 items-end gap-2 sm:grid-cols-[minmax(0,1fr)_auto_auto]">
                <Field id=field_id label="Riot ID" error=Signal::derive(move || error.get())>
                    <TextField value=riot_id placeholder="Name#Tag" />
                </Field>
                <Button on_click=refresh loading=Signal::derive(move || fetching.get())>"Refresh"</Button>
                <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| confirm.set(true))>"Remove"</Button>
            </div>
            <ConfirmDialog
                id="remove-player"
                open=confirm
                title="Remove this player?"
                message="The player and their scouted games leave the dossier."
                confirm_label="Remove player"
                tone=ConfirmTone::Destructive
                on_confirm=remove
            />
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_line_counts_players_and_champions() {
        assert_eq!(evidence_line(0, 4), "No players yet");
        assert_eq!(evidence_line(1, 1), "1 player, 1 champion seen");
        assert_eq!(evidence_line(5, 0), "5 players, no champions seen");
        assert_eq!(evidence_line(5, 100), "5 players, 100 champions seen");
    }

    #[test]
    fn surveyed_line_flags_stale_dossiers() {
        assert_eq!(surveyed_line(None, false), "never surveyed");
        assert_eq!(
            surveyed_line(Some("2 days ago"), false),
            "surveyed 2 days ago"
        );
        assert_eq!(
            surveyed_line(Some("9 days ago"), true),
            "surveyed 9 days ago, over a week"
        );
    }
}
