use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;
use surrealdb::{engine::local::Db, types::{RecordId, SurrealValue, ToSql}, Surreal};
use thiserror::Error;

use crate::models::{
    champion::ChampionPoolEntry,
    draft::{Draft, DraftAction},
    game_plan::{GamePlan, PostGameLearning},
    match_data::PlayerMatchStats,
    team::Team,
    user::{JoinRequest, TeamMember},
};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::types::Error),
    #[error("Record not found")]
    NotFound,
    #[error("{0}")]
    Other(String),
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Deserialize, SurrealValue)]
struct IdRecord {
    id: RecordId,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct TeamRef {
    team: RecordId,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeam {
    id: RecordId,
    name: String,
    region: String,
    created_by: RecordId,
}

impl From<DbTeam> for Team {
    fn from(t: DbTeam) -> Self {
        Team {
            id: Some(t.id.to_sql()),
            name: t.name,
            region: t.region,
            created_by: t.created_by.to_sql(),
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeamMember {
    user_id: RecordId,
    username: String,
    role: String,
    roster_type: String,
    riot_summoner_name: Option<String>,
}

impl From<DbTeamMember> for TeamMember {
    fn from(m: DbTeamMember) -> Self {
        TeamMember {
            user_id: m.user_id.to_sql(),
            username: m.username,
            role: m.role,
            roster_type: m.roster_type,
            riot_summoner_name: m.riot_summoner_name,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbJoinRequest {
    id: RecordId,
    team: RecordId,
    user_id: RecordId,
    username: String,
    riot_summoner_name: Option<String>,
}

impl From<DbJoinRequest> for JoinRequest {
    fn from(r: DbJoinRequest) -> Self {
        JoinRequest {
            id: r.id.to_sql(),
            team_id: r.team.to_sql(),
            user_id: r.user_id.to_sql(),
            username: r.username,
            riot_summoner_name: r.riot_summoner_name,
        }
    }
}

pub async fn init_db(path: &str) -> DbResult<Arc<Surreal<Db>>> {
    use surrealdb::engine::local::SurrealKv;

    let db = Surreal::new::<SurrealKv>(path).await?;
    db.use_ns("lol_companion").use_db("app").await?;
    apply_schema(&db).await?;
    Ok(Arc::new(db))
}

async fn apply_schema(db: &Surreal<Db>) -> DbResult<()> {
    db.query(include_str!("../../schema.surql")).await?.check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// User
// ---------------------------------------------------------------------------

pub async fn create_user(
    db: &Surreal<Db>,
    username: String,
    email: String,
    password_hash: String,
) -> DbResult<String> {
    let mut response = db
        .query("CREATE user SET username = $username, email = $email, password_hash = $password_hash")
        .bind(("username", username))
        .bind(("email", email))
        .bind(("password_hash", password_hash))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create user".into())),
    }
}

pub async fn update_user_riot(db: &Surreal<Db>, user_id: String, puuid: String, summoner_name: String) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(&user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET riot_puuid = $puuid, riot_summoner_name = $name")
        .bind(("user_key", user_key))
        .bind(("puuid", puuid))
        .bind(("name", summoner_name))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Champion Pool
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbPoolEntry {
    id: RecordId,
    user: RecordId,
    champion: String,
    role: String,
}

impl From<DbPoolEntry> for ChampionPoolEntry {
    fn from(e: DbPoolEntry) -> Self {
        ChampionPoolEntry {
            id: Some(e.id.to_sql()),
            user_id: e.user.to_sql(),
            champion: e.champion,
            role: e.role,
        }
    }
}

pub async fn get_champion_pool(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<ChampionPoolEntry>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT * FROM champion_pool WHERE user = type::record('user', $user_key) ORDER BY role, champion ASC")
        .bind(("user_key", user_key))
        .await?;
    let entries: Vec<DbPoolEntry> = r.take(0).unwrap_or_default();
    Ok(entries.into_iter().map(ChampionPoolEntry::from).collect())
}

pub async fn add_to_champion_pool(db: &Surreal<Db>, user_id: &str, champion: String, role: String) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    // Skip if already exists
    let mut check = db
        .query("SELECT id FROM champion_pool WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role LIMIT 1")
        .bind(("user_key", user_key.clone()))
        .bind(("champion", champion.clone()))
        .bind(("role", role.clone()))
        .await?;
    let existing: Option<IdRecord> = check.take(0)?;
    if existing.is_some() {
        return Ok(());
    }
    db.query("CREATE champion_pool SET user = type::record('user', $user_key), champion = $champion, role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion))
        .bind(("role", role))
        .await?
        .check()?;
    Ok(())
}

pub async fn remove_from_champion_pool(db: &Surreal<Db>, user_id: &str, champion: String, role: String) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("DELETE champion_pool WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion))
        .bind(("role", role))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Teams
// ---------------------------------------------------------------------------

pub async fn create_team(
    db: &Surreal<Db>,
    user_id: &str,
    name: String,
    region: String,
) -> DbResult<String> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let mut response = db
        .query("CREATE team SET name = $name, region = $region, created_by = type::record('user', $user_key)")
        .bind(("name", name))
        .bind(("region", region))
        .bind(("user_key", user_key.clone()))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    let team_id = match row {
        Some(r) => r.id.to_sql(),
        None => return Err(DbError::Other("Failed to create team".into())),
    };

    let team_key = team_id.strip_prefix("team:").unwrap_or(&team_id).to_string();

    db.query("CREATE team_member SET team = type::record('team', $team_key), user = type::record('user', $user_key), role = 'sub'")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()?;

    Ok(team_id)
}

pub async fn get_user_team_id(db: &Surreal<Db>, user_id: &str) -> DbResult<Option<String>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut result = db
        .query("SELECT team FROM team_member WHERE user = type::record('user', $user_key) LIMIT 1")
        .bind(("user_key", user_key))
        .await?;

    let row: Option<TeamRef> = result.take(0)?;
    Ok(row.map(|r| r.team.to_sql()))
}

pub async fn update_team(db: &Surreal<Db>, team_id: &str, name: String, region: String) -> DbResult<()> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    db.query("UPDATE type::record('team', $team_key) SET name=$name, region=$region")
        .bind(("team_key", team_key))
        .bind(("name", name))
        .bind(("region", region))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_member_role(db: &Surreal<Db>, team_id: &str, user_id: &str, role: String) -> DbResult<()> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE team_member SET role=$role WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key)")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("role", role))
        .await?
        .check()?;
    Ok(())
}

pub async fn remove_team_member(db: &Surreal<Db>, team_id: &str, user_id: &str) -> DbResult<()> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("DELETE team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key)")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn list_all_teams(db: &Surreal<Db>) -> DbResult<Vec<Team>> {
    let mut r = db.query("SELECT * FROM team ORDER BY name ASC").await?;
    let teams: Vec<DbTeam> = r.take(0).unwrap_or_default();
    Ok(teams.into_iter().map(Team::from).collect())
}

pub async fn join_team(db: &Surreal<Db>, user_id: &str, team_id: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    // Prevent duplicate membership
    let mut check = db
        .query("SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) LIMIT 1")
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .await?;
    let existing: Option<IdRecord> = check.take(0)?;
    if existing.is_some() {
        return Err(DbError::Other("You are already a member of this team".into()));
    }
    db.query("CREATE team_member SET team = type::record('team', $team_key), user = type::record('user', $user_key), role = 'sub'")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_user_teams(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<Team>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut result = db
        .query("SELECT team FROM team_member WHERE user = type::record('user', $user_key)")
        .bind(("user_key", user_key))
        .await?;
    let refs: Vec<TeamRef> = result.take(0).unwrap_or_default();
    let mut teams = Vec::new();
    for r in refs {
        let id_sql = r.team.to_sql();
        let team_key = id_sql.strip_prefix("team:").unwrap_or(&id_sql).to_string();
        let mut tr = db
            .query("SELECT * FROM type::record('team', $team_key)")
            .bind(("team_key", team_key))
            .await?;
        let db_team: Option<DbTeam> = tr.take(0)?;
        if let Some(t) = db_team {
            teams.push(Team::from(t));
        }
    }
    Ok(teams)
}

pub async fn get_user_team_with_members(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Option<(Team, Vec<TeamMember>)>> {
    let team_id = match get_user_team_id(db, user_id).await? {
        Some(id) => id,
        None => return Ok(None),
    };

    let team_key = team_id.strip_prefix("team:").unwrap_or(&team_id).to_string();

    let mut team_result = db
        .query("SELECT * FROM type::record('team', $team_key)")
        .bind(("team_key", team_key.clone()))
        .await?;
    let db_team: Option<DbTeam> = team_result.take(0)?;
    let team: Option<Team> = db_team.map(Team::from);

    let team = match team {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut members_result = db
        .query("SELECT user.username as username, user.id as user_id, role, roster_type, user.riot_summoner_name as riot_summoner_name FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await?;
    let db_members: Vec<DbTeamMember> = members_result.take(0).unwrap_or_default();
    let members: Vec<TeamMember> = db_members.into_iter().map(TeamMember::from).collect();

    Ok(Some((team, members)))
}

pub async fn create_join_request(db: &Surreal<Db>, user_id: &str, team_id: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    // Prevent duplicate pending requests
    let mut check = db
        .query("SELECT id FROM join_request WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) AND status = 'pending' LIMIT 1")
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .await?;
    let existing: Option<IdRecord> = check.take(0)?;
    if existing.is_some() {
        return Err(DbError::Other("You already have a pending request for this team".into()));
    }
    // Also prevent joining a team you're already in
    let mut member_check = db
        .query("SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) LIMIT 1")
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .await?;
    let already_member: Option<IdRecord> = member_check.take(0)?;
    if already_member.is_some() {
        return Err(DbError::Other("You are already a member of this team".into()));
    }
    db.query("CREATE join_request SET team = type::record('team', $team_key), user = type::record('user', $user_key), status = 'pending'")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn list_pending_join_requests(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<JoinRequest>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT id, team, user.id as user_id, user.username as username, user.riot_summoner_name as riot_summoner_name FROM join_request WHERE team = type::record('team', $team_key) AND status = 'pending' ORDER BY created_at ASC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbJoinRequest> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(JoinRequest::from).collect())
}

pub async fn respond_to_join_request(db: &Surreal<Db>, request_id: &str, accept: bool, team_id: &str) -> DbResult<()> {
    let req_key = request_id.strip_prefix("join_request:").unwrap_or(request_id).to_string();
    let status = if accept { "accepted" } else { "declined" };
    db.query("UPDATE type::record('join_request', $req_key) SET status = $status")
        .bind(("req_key", req_key.clone()))
        .bind(("status", status.to_string()))
        .await?
        .check()?;
    if accept {
        // Get the user from the request and add them to the team
        let mut r = db
            .query("SELECT user FROM type::record('join_request', $req_key)")
            .bind(("req_key", req_key))
            .await?;
        #[derive(Debug, serde::Deserialize, SurrealValue)]
        struct UserRef { user: RecordId }
        let row: Option<UserRef> = r.take(0)?;
        if let Some(ur) = row {
            let user_sql = ur.user.to_sql();
            let user_key = user_sql.strip_prefix("user:").unwrap_or(&user_sql).to_string();
            let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
            db.query("CREATE team_member SET team = type::record('team', $team_key), user = type::record('user', $user_key), role = 'unassigned', roster_type = 'sub'")
                .bind(("team_key", team_key))
                .bind(("user_key", user_key))
                .await?
                .check()?;
        }
    }
    Ok(())
}

pub async fn count_pending_join_requests(db: &Surreal<Db>, team_id: &str) -> DbResult<usize> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT count() as n FROM join_request WHERE team = type::record('team', $team_key) AND status = 'pending' GROUP ALL")
        .bind(("team_key", team_key))
        .await?;
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Count { n: i64 }
    let row: Option<Count> = r.take(0)?;
    Ok(row.map(|c| c.n as usize).unwrap_or(0))
}

pub async fn assign_to_slot(db: &Surreal<Db>, team_id: &str, user_id: &str, role: &str) -> DbResult<()> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    // Move existing starter for that role to sub
    db.query("UPDATE team_member SET roster_type = 'sub' WHERE team = type::record('team', $team_key) AND role = $role AND roster_type = 'starter'")
        .bind(("team_key", team_key.clone()))
        .bind(("role", role.to_string()))
        .await?
        .check()?;
    // Assign this user as starter
    db.query("UPDATE team_member SET role = $role, roster_type = 'starter' WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key)")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("role", role.to_string()))
        .await?
        .check()?;
    Ok(())
}

pub async fn remove_from_slot(db: &Surreal<Db>, team_id: &str, user_id: &str) -> DbResult<()> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE team_member SET roster_type = 'sub' WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key)")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Drafts
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbDraft {
    id: RecordId,
    name: String,
    team: RecordId,
    created_by: RecordId,
    opponent: Option<String>,
    notes: Option<String>,
    rating: Option<String>,
    comments: Vec<String>,
}

impl From<DbDraft> for Draft {
    fn from(d: DbDraft) -> Self {
        Draft {
            id: Some(d.id.to_sql()),
            name: d.name,
            team_id: d.team.to_sql(),
            created_by: d.created_by.to_sql(),
            opponent: d.opponent,
            notes: d.notes,
            rating: d.rating,
            actions: Vec::new(),
            comments: d.comments,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbDraftAction {
    id: RecordId,
    draft: RecordId,
    phase: String,
    side: String,
    champion: String,
    order: i32,
}

impl From<DbDraftAction> for DraftAction {
    fn from(a: DbDraftAction) -> Self {
        DraftAction {
            id: Some(a.id.to_sql()),
            draft_id: a.draft.to_sql(),
            phase: a.phase,
            side: a.side,
            champion: a.champion,
            order: a.order,
        }
    }
}

pub async fn save_draft(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    name: String,
    opponent: Option<String>,
    notes: Option<String>,
    comments: Vec<String>,
    actions: Vec<DraftAction>,
    rating: Option<String>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let mut response = db
        .query("CREATE draft SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent = $opponent, notes = $notes, comments = $comments, rating = $rating")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
        .bind(("comments", comments))
        .bind(("rating", rating))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    let draft_id = match row {
        Some(r) => r.id.to_sql(),
        None => return Err(DbError::Other("Failed to create draft".into())),
    };

    let draft_key = draft_id.strip_prefix("draft:").unwrap_or(&draft_id).to_string();

    for action in actions {
        let dk = draft_key.clone();
        db.query("CREATE draft_action SET draft = type::record('draft', $draft_key), phase = $phase, side = $side, champion = $champion, `order` = $order")
            .bind(("draft_key", dk))
            .bind(("phase", action.phase))
            .bind(("side", action.side))
            .bind(("champion", action.champion))
            .bind(("order", action.order))
            .await?
            .check()?;
    }

    Ok(draft_id)
}

pub async fn list_drafts(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<Draft>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut result = db
        .query("SELECT * FROM draft WHERE team = type::record('team', $team_key) ORDER BY created_at DESC; SELECT * FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE team = type::record('team', $team_key)) ORDER BY `order` ASC")
        .bind(("team_key", team_key))
        .await?;
    let db_drafts: Vec<DbDraft> = result.take(0).unwrap_or_default();
    let db_actions: Vec<DbDraftAction> = result.take(1).unwrap_or_default();

    let mut actions_by_draft: HashMap<String, Vec<DraftAction>> = HashMap::new();
    for a in db_actions {
        let draft_id = a.draft.to_sql();
        actions_by_draft.entry(draft_id).or_default().push(DraftAction::from(a));
    }

    Ok(db_drafts.into_iter().map(|d| {
        let id = d.id.to_sql();
        let mut draft = Draft::from(d);
        draft.actions = actions_by_draft.remove(&id).unwrap_or_default();
        draft
    }).collect())
}

pub async fn update_draft(
    db: &Surreal<Db>,
    draft_id: &str,
    name: String,
    opponent: Option<String>,
    notes: Option<String>,
    comments: Vec<String>,
    actions: Vec<DraftAction>,
    rating: Option<String>,
) -> DbResult<()> {
    let draft_key = draft_id.strip_prefix("draft:").unwrap_or(draft_id).to_string();

    db.query("UPDATE type::record('draft', $draft_key) SET name=$name, opponent=$opponent, notes=$notes, comments=$comments, rating=$rating")
        .bind(("draft_key", draft_key.clone()))
        .bind(("name", name))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
        .bind(("comments", comments))
        .bind(("rating", rating))
        .await?
        .check()?;

    db.query("DELETE draft_action WHERE draft = type::record('draft', $draft_key)")
        .bind(("draft_key", draft_key.clone()))
        .await?
        .check()?;

    for action in actions {
        let dk = draft_key.clone();
        db.query("CREATE draft_action SET draft = type::record('draft', $draft_key), phase = $phase, side = $side, champion = $champion, `order` = $order")
            .bind(("draft_key", dk))
            .bind(("phase", action.phase))
            .bind(("side", action.side))
            .bind(("champion", action.champion))
            .bind(("order", action.order))
            .await?
            .check()?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Stats / Matches
// ---------------------------------------------------------------------------

pub async fn get_player_stats(db: &Surreal<Db>, user_id: &str) -> DbResult<Vec<PlayerMatchStats>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut result = db
        .query("SELECT * FROM player_match WHERE user = type::record('user', $user_key) LIMIT 50")
        .bind(("user_key", user_key))
        .await?;
    Ok(result.take(0).unwrap_or_default())
}

pub async fn store_matches(
    db: &Surreal<Db>,
    user_id: &str,
    matches: Vec<crate::server::riot::MatchData>,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    for m in matches {
        let mut r = db
            .query("SELECT id FROM match WHERE match_id = $match_id LIMIT 1")
            .bind(("match_id", m.match_id.clone()))
            .await?;

        let existing: Option<IdRecord> = r.take(0)?;
        let match_key = if let Some(rec) = existing {
            let id_str = rec.id.to_sql();
            id_str.strip_prefix("match:").unwrap_or(&id_str).to_string()
        } else {
            let mut cr = db
                .query("CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration")
                .bind(("match_id", m.match_id.clone()))
                .bind(("queue_id", m.queue_id))
                .bind(("game_duration", m.game_duration))
                .await?;
            let row: Option<IdRecord> = cr.take(0)?;
            match row {
                Some(rec) => {
                    let id_str = rec.id.to_sql();
                    id_str.strip_prefix("match:").unwrap_or(&id_str).to_string()
                }
                None => continue,
            }
        };

        db.query("CREATE player_match SET match = type::record('match', $match_key), user = type::record('user', $user_key), champion = $champion, kills = $kills, deaths = $deaths, assists = $assists, cs = $cs, vision_score = $vision_score, damage = $damage, win = $win")
            .bind(("match_key", match_key))
            .bind(("user_key", user_key.clone()))
            .bind(("champion", m.champion))
            .bind(("kills", m.kills))
            .bind(("deaths", m.deaths))
            .bind(("assists", m.assists))
            .bind(("cs", m.cs))
            .bind(("vision_score", m.vision_score))
            .bind(("damage", m.damage))
            .bind(("win", m.win))
            .await
            .ok();
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Game plans / post-game
// ---------------------------------------------------------------------------

pub async fn save_game_plan(db: &Surreal<Db>, plan: GamePlan) -> DbResult<()> {
    let team_key = plan.team_id.strip_prefix("team:").unwrap_or(&plan.team_id).to_string();
    db.query("CREATE game_plan SET team = type::record('team', $team_key), draft = $draft_id, win_conditions = $win_conditions, objective_priority = $objective_priority, teamfight_strategy = $teamfight_strategy, early_game = $early_game, notes = $notes")
        .bind(("team_key", team_key))
        .bind(("draft_id", plan.draft_id))
        .bind(("win_conditions", plan.win_conditions))
        .bind(("objective_priority", plan.objective_priority))
        .bind(("teamfight_strategy", plan.teamfight_strategy))
        .bind(("early_game", plan.early_game))
        .bind(("notes", plan.notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn save_post_game_learning(db: &Surreal<Db>, learning: PostGameLearning) -> DbResult<()> {
    let team_key = learning.team_id.strip_prefix("team:").unwrap_or(&learning.team_id).to_string();
    let created_by_key = learning.created_by.strip_prefix("user:").unwrap_or(&learning.created_by).to_string();
    db.query("CREATE post_game_learning SET team = type::record('team', $team_key), match = $match_id, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, created_by = type::record('user', $created_by_key)")
        .bind(("team_key", team_key))
        .bind(("match_id", learning.match_id))
        .bind(("what_went_well", learning.what_went_well))
        .bind(("improvements", learning.improvements))
        .bind(("action_items", learning.action_items))
        .bind(("created_by_key", created_by_key))
        .await?
        .check()?;
    Ok(())
}
