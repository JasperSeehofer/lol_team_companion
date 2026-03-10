use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;
use surrealdb::{engine::local::Db, types::{RecordId, SurrealValue, ToSql}, Surreal};
use thiserror::Error;

use crate::models::{
    champion::ChampionPoolEntry,
    draft::{Draft, DraftAction, DraftTree, DraftTreeNode},
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
// Draft Trees
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbDraftTree {
    id: RecordId,
    name: String,
    team: RecordId,
    created_by: RecordId,
    opponent: Option<String>,
}

impl From<DbDraftTree> for DraftTree {
    fn from(t: DbDraftTree) -> Self {
        DraftTree {
            id: Some(t.id.to_sql()),
            name: t.name,
            team_id: t.team.to_sql(),
            created_by: t.created_by.to_sql(),
            opponent: t.opponent,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbDraftTreeNode {
    id: RecordId,
    tree: RecordId,
    parent: Option<RecordId>,
    label: String,
    notes: Option<String>,
    is_improvised: bool,
    sort_order: i32,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTreeNodeAction {
    id: RecordId,
    node: RecordId,
    phase: String,
    side: String,
    champion: String,
    order: i32,
}

pub async fn create_draft_tree(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    name: String,
    opponent: Option<String>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let mut response = db
        .query("CREATE draft_tree SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent = $opponent")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("opponent", opponent))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => {
            let tree_id = r.id.to_sql();
            // Auto-create a root node
            let tree_key = tree_id.strip_prefix("draft_tree:").unwrap_or(&tree_id).to_string();
            db.query("CREATE draft_tree_node SET tree = type::record('draft_tree', $tree_key), parent = NONE, label = 'Root', sort_order = 0")
                .bind(("tree_key", tree_key))
                .await?
                .check()?;
            Ok(tree_id)
        }
        None => Err(DbError::Other("Failed to create draft tree".into())),
    }
}

pub async fn list_draft_trees(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<DraftTree>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT * FROM draft_tree WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;
    let trees: Vec<DbDraftTree> = r.take(0).unwrap_or_default();
    Ok(trees.into_iter().map(DraftTree::from).collect())
}

pub async fn delete_draft_tree(db: &Surreal<Db>, tree_id: &str) -> DbResult<()> {
    let tree_key = tree_id.strip_prefix("draft_tree:").unwrap_or(tree_id).to_string();
    // Delete actions, then nodes, then tree
    db.query("DELETE tree_node_action WHERE node IN (SELECT VALUE id FROM draft_tree_node WHERE tree = type::record('draft_tree', $tree_key)); DELETE draft_tree_node WHERE tree = type::record('draft_tree', $tree_key); DELETE type::record('draft_tree', $tree_key)")
        .bind(("tree_key", tree_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_draft_tree(
    db: &Surreal<Db>,
    tree_id: &str,
    name: String,
    opponent: Option<String>,
) -> DbResult<()> {
    let tree_key = tree_id.strip_prefix("draft_tree:").unwrap_or(tree_id).to_string();
    db.query("UPDATE type::record('draft_tree', $tree_key) SET name = $name, opponent = $opponent")
        .bind(("tree_key", tree_key))
        .bind(("name", name))
        .bind(("opponent", opponent))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_tree_nodes(db: &Surreal<Db>, tree_id: &str) -> DbResult<Vec<DraftTreeNode>> {
    let tree_key = tree_id.strip_prefix("draft_tree:").unwrap_or(tree_id).to_string();

    let mut result = db
        .query("SELECT * FROM draft_tree_node WHERE tree = type::record('draft_tree', $tree_key) ORDER BY sort_order ASC; SELECT * FROM tree_node_action WHERE node IN (SELECT VALUE id FROM draft_tree_node WHERE tree = type::record('draft_tree', $tree_key)) ORDER BY `order` ASC")
        .bind(("tree_key", tree_key))
        .await?;

    let db_nodes: Vec<DbDraftTreeNode> = result.take(0).unwrap_or_default();
    let db_actions: Vec<DbTreeNodeAction> = result.take(1).unwrap_or_default();

    // Group actions by node
    let mut actions_by_node: HashMap<String, Vec<DraftAction>> = HashMap::new();
    for a in db_actions {
        let node_id = a.node.to_sql();
        actions_by_node.entry(node_id).or_default().push(DraftAction {
            id: Some(a.id.to_sql()),
            draft_id: String::new(),
            phase: a.phase,
            side: a.side,
            champion: a.champion,
            order: a.order,
        });
    }

    // Build flat list of nodes
    let flat_nodes: Vec<DraftTreeNode> = db_nodes
        .into_iter()
        .map(|n| {
            let id = n.id.to_sql();
            let actions = actions_by_node.remove(&id).unwrap_or_default();
            DraftTreeNode {
                id: Some(id),
                tree_id: n.tree.to_sql(),
                parent_id: n.parent.map(|p| p.to_sql()),
                label: n.label,
                notes: n.notes,
                is_improvised: n.is_improvised,
                sort_order: n.sort_order,
                actions,
                children: Vec::new(),
            }
        })
        .collect();

    // Build tree structure
    let mut node_map: HashMap<String, DraftTreeNode> = HashMap::new();
    let mut root_ids: Vec<String> = Vec::new();
    let mut child_ids: Vec<(String, String)> = Vec::new(); // (parent_id, child_id)

    for node in flat_nodes {
        let id = node.id.clone().unwrap_or_default();
        if node.parent_id.is_none() {
            root_ids.push(id.clone());
        } else {
            child_ids.push((node.parent_id.clone().unwrap_or_default(), id.clone()));
        }
        node_map.insert(id, node);
    }

    // Attach children bottom-up
    // Sort child_ids so we process deepest nodes first (approximation: reverse order)
    child_ids.reverse();
    for (parent_id, child_id) in child_ids {
        if let Some(child) = node_map.remove(&child_id) {
            if let Some(parent) = node_map.get_mut(&parent_id) {
                parent.children.push(child);
            } else {
                // Parent was already consumed, put child back
                node_map.insert(child_id, child);
            }
        }
    }

    // Sort children by sort_order
    fn sort_children(node: &mut DraftTreeNode) {
        node.children.sort_by_key(|c| c.sort_order);
        for child in &mut node.children {
            sort_children(child);
        }
    }

    let mut roots: Vec<DraftTreeNode> = root_ids
        .into_iter()
        .filter_map(|id| node_map.remove(&id))
        .collect();

    for root in &mut roots {
        sort_children(root);
    }

    Ok(roots)
}

pub async fn create_tree_node(
    db: &Surreal<Db>,
    tree_id: &str,
    parent_id: Option<String>,
    label: String,
) -> DbResult<String> {
    let tree_key = tree_id.strip_prefix("draft_tree:").unwrap_or(tree_id).to_string();

    let parent_clause = match &parent_id {
        Some(pid) => {
            let pk = pid.strip_prefix("draft_tree_node:").unwrap_or(pid).to_string();
            format!("parent = type::record('draft_tree_node', '{pk}')")
        }
        None => "parent = NONE".to_string(),
    };

    // Get next sort_order among siblings
    let sort_query = match &parent_id {
        Some(pid) => {
            let pk = pid.strip_prefix("draft_tree_node:").unwrap_or(pid).to_string();
            format!("SELECT count() as n FROM draft_tree_node WHERE tree = type::record('draft_tree', '{tree_key}') AND parent = type::record('draft_tree_node', '{pk}') GROUP ALL")
        }
        None => format!("SELECT count() as n FROM draft_tree_node WHERE tree = type::record('draft_tree', '{tree_key}') AND parent = NONE GROUP ALL"),
    };

    #[derive(Debug, Deserialize, SurrealValue)]
    struct Count { n: i64 }

    let mut sr = db.query(&sort_query).await?;
    let count: Option<Count> = sr.take(0)?;
    let sort_order = count.map(|c| c.n as i32).unwrap_or(0);

    let query = format!(
        "CREATE draft_tree_node SET tree = type::record('draft_tree', $tree_key), {parent_clause}, label = $label, sort_order = $sort_order"
    );
    let mut response = db
        .query(&query)
        .bind(("tree_key", tree_key))
        .bind(("label", label))
        .bind(("sort_order", sort_order))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create tree node".into())),
    }
}

pub async fn update_tree_node(
    db: &Surreal<Db>,
    node_id: &str,
    label: String,
    notes: Option<String>,
    is_improvised: bool,
    actions: Vec<DraftAction>,
) -> DbResult<()> {
    let node_key = node_id.strip_prefix("draft_tree_node:").unwrap_or(node_id).to_string();

    db.query("UPDATE type::record('draft_tree_node', $node_key) SET label = $label, notes = $notes, is_improvised = $is_improvised")
        .bind(("node_key", node_key.clone()))
        .bind(("label", label))
        .bind(("notes", notes))
        .bind(("is_improvised", is_improvised))
        .await?
        .check()?;

    // Replace actions
    db.query("DELETE tree_node_action WHERE node = type::record('draft_tree_node', $node_key)")
        .bind(("node_key", node_key.clone()))
        .await?
        .check()?;

    for action in actions {
        let nk = node_key.clone();
        db.query("CREATE tree_node_action SET node = type::record('draft_tree_node', $node_key), phase = $phase, side = $side, champion = $champion, `order` = $order")
            .bind(("node_key", nk))
            .bind(("phase", action.phase))
            .bind(("side", action.side))
            .bind(("champion", action.champion))
            .bind(("order", action.order))
            .await?
            .check()?;
    }

    Ok(())
}

pub async fn delete_tree_node(db: &Surreal<Db>, node_id: &str) -> DbResult<()> {
    let node_key = node_id.strip_prefix("draft_tree_node:").unwrap_or(node_id).to_string();
    // Delete actions for this node and all descendants, then delete nodes
    // First get all descendant node IDs
    // SurrealDB doesn't have recursive CTEs, so we do iterative deletion
    let mut to_delete = vec![node_key.clone()];
    let mut i = 0;
    while i < to_delete.len() {
        let key = to_delete[i].clone();
        let mut r = db
            .query("SELECT id FROM draft_tree_node WHERE parent = type::record('draft_tree_node', $key)")
            .bind(("key", key))
            .await?;
        let children: Vec<IdRecord> = r.take(0).unwrap_or_default();
        for child in children {
            let child_id = child.id.to_sql();
            let child_key = child_id.strip_prefix("draft_tree_node:").unwrap_or(&child_id).to_string();
            to_delete.push(child_key);
        }
        i += 1;
    }

    for key in to_delete {
        db.query("DELETE tree_node_action WHERE node = type::record('draft_tree_node', $key); DELETE type::record('draft_tree_node', $key)")
            .bind(("key", key))
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
            // Convert epoch ms to ISO datetime for SurrealDB
            let game_end_str = m.game_end_epoch_ms.map(|ms| {
                let secs = ms / 1000;
                let nanos = ((ms % 1000) * 1_000_000) as u32;
                chrono::DateTime::from_timestamp(secs, nanos)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            });

            let query = match &game_end_str {
                Some(ge) => format!(
                    "CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration, game_end = <datetime>'{ge}'"
                ),
                None => "CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration".to_string(),
            };

            let mut cr = db
                .query(&query)
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

        // Check if player_match already exists for this user+match
        let mut check = db
            .query("SELECT id FROM player_match WHERE match = type::record('match', $match_key) AND user = type::record('user', $user_key) LIMIT 1")
            .bind(("match_key", match_key.clone()))
            .bind(("user_key", user_key.clone()))
            .await?;
        let existing_pm: Option<IdRecord> = check.take(0)?;
        if existing_pm.is_some() {
            continue; // Already stored
        }

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

/// Get team stats with player details, joining match + player_match.
/// Returns all player_match entries for team members, enriched with match date.
pub async fn get_team_match_stats(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<TeamMatchRow>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Get all team member user IDs
    let mut r = db
        .query("SELECT user FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct UserRef { user: RecordId }
    let user_refs: Vec<UserRef> = r.take(0).unwrap_or_default();
    if user_refs.is_empty() {
        return Ok(Vec::new());
    }

    // Build a query that gets all player_match records for team members with match info
    // Using a join via the match record
    let user_ids: Vec<String> = user_refs.iter().map(|u| u.user.to_sql()).collect();
    let user_id_list = user_ids.iter()
        .map(|id| format!("{id}"))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "SELECT *, match.match_id as riot_match_id, match.game_duration as game_duration, match.game_end as game_end, user.username as username FROM player_match WHERE user IN [{user_id_list}] ORDER BY match.game_end DESC LIMIT 200"
    );

    let mut result = db.query(&query).await?;
    let rows: Vec<DbTeamMatchRow> = result.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(TeamMatchRow::from).collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeamMatchRow {
    id: RecordId,
    #[serde(rename = "match")]
    match_ref: RecordId,
    user: RecordId,
    username: String,
    riot_match_id: String,
    game_duration: i32,
    game_end: Option<String>,
    champion: String,
    kills: i32,
    deaths: i32,
    assists: i32,
    cs: i32,
    vision_score: i32,
    damage: i32,
    win: bool,
}

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

impl From<DbTeamMatchRow> for TeamMatchRow {
    fn from(r: DbTeamMatchRow) -> Self {
        TeamMatchRow {
            match_db_id: r.match_ref.to_sql(),
            user_id: r.user.to_sql(),
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
        }
    }
}

/// Get roster member puuids for syncing
pub async fn get_roster_puuids(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<(String, String, Option<String>)>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT user.id as user_id, user.username as username, user.riot_puuid as riot_puuid FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberPuuid {
        user_id: RecordId,
        username: String,
        riot_puuid: Option<String>,
    }

    let members: Vec<MemberPuuid> = r.take(0).unwrap_or_default();
    Ok(members.into_iter().map(|m| (m.user_id.to_sql(), m.username, m.riot_puuid)).collect())
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
