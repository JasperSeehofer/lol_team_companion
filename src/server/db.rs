use std::sync::Arc;
use serde::Deserialize;
use surrealdb::{engine::local::Db, types::{RecordId, SurrealValue, ToSql}, Surreal};
use thiserror::Error;

use crate::models::{
    draft::{Draft, DraftAction},
    game_plan::{GamePlan, PostGameLearning},
    match_data::PlayerMatchStats,
    team::Team,
    user::TeamMember,
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
    riot_summoner_name: Option<String>,
}

impl From<DbTeamMember> for TeamMember {
    fn from(m: DbTeamMember) -> Self {
        TeamMember {
            user_id: m.user_id.to_sql(),
            username: m.username,
            role: m.role,
            riot_summoner_name: m.riot_summoner_name,
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
        .query("SELECT user.username as username, user.id as user_id, role, user.riot_summoner_name as riot_summoner_name FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await?;
    let db_members: Vec<DbTeamMember> = members_result.take(0).unwrap_or_default();
    let members: Vec<TeamMember> = db_members.into_iter().map(TeamMember::from).collect();

    Ok(Some((team, members)))
}

// ---------------------------------------------------------------------------
// Drafts
// ---------------------------------------------------------------------------

pub async fn save_draft(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    name: String,
    opponent: Option<String>,
    notes: Option<String>,
    actions: Vec<DraftAction>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let mut response = db
        .query("CREATE draft SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent = $opponent, notes = $notes")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
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
        .query("SELECT * FROM draft WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;
    Ok(result.take(0).unwrap_or_default())
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
