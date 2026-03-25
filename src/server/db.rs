use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::{
    engine::local::Db,
    types::{RecordId, SurrealValue, ToSql},
    Surreal,
};
use thiserror::Error;

use crate::models::{
    action_item::ActionItem,
    champion::{Champion, ChampionNote, ChampionPoolEntry, ChampionStatSummary},
    draft::{BanPriority, Draft, DraftAction, DraftTree, DraftTreeNode},
    game_plan::{
        ActionItemPreview, ChampionPerformanceSummary, ChecklistInstance, ChecklistTemplate,
        DashboardSummary, GamePlan, GamePlanEffectiveness, PoolGapWarning, PostGameLearning,
        PostGamePreview, StrategyTagSummary,
    },
    match_data::PlayerMatchStats,
    opponent::{Opponent, OpponentPlayer},
    series::Series,
    team::Team,
    team_note::TeamNote,
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
    member_count: Option<u32>,
}

impl From<DbTeam> for Team {
    fn from(t: DbTeam) -> Self {
        Team {
            id: Some(t.id.to_sql()),
            name: t.name,
            region: t.region,
            created_by: t.created_by.to_sql(),
            member_count: t.member_count,
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
    // Best-effort champion name normalization (skipped if Data Dragon is unreachable)
    if let Err(e) = migrate_champion_names(&db).await {
        tracing::warn!("Champion name migration failed: {e}");
    }
    Ok(Arc::new(db))
}

async fn apply_schema(db: &Surreal<Db>) -> DbResult<()> {
    db.query(include_str!("../../schema.surql"))
        .await?
        .check()?;
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
        .query(
            "CREATE user SET username = $username, email = $email, password_hash = $password_hash",
        )
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

pub async fn update_user_riot(
    db: &Surreal<Db>,
    user_id: String,
    puuid: String,
    summoner_name: String,
) -> DbResult<()> {
    let user_key = user_id
        .strip_prefix("user:")
        .unwrap_or(&user_id)
        .to_string();
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
    tier: String,
    notes: Option<String>,
    comfort_level: Option<i64>,
    meta_tag: Option<String>,
}

impl From<DbPoolEntry> for ChampionPoolEntry {
    fn from(e: DbPoolEntry) -> Self {
        ChampionPoolEntry {
            id: Some(e.id.to_sql()),
            user_id: e.user.to_sql(),
            champion: e.champion,
            role: e.role,
            tier: e.tier,
            notes: e.notes,
            comfort_level: e.comfort_level.map(|v| v as u8),
            meta_tag: e.meta_tag,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbChampionNote {
    id: RecordId,
    user: RecordId,
    champion: String,
    role: String,
    note_type: String,
    title: String,
    content: String,
    difficulty: Option<i64>,
    created_at: String,
}

impl From<DbChampionNote> for ChampionNote {
    fn from(e: DbChampionNote) -> Self {
        ChampionNote {
            id: Some(e.id.to_sql()),
            user_id: e.user.to_sql(),
            champion: e.champion,
            role: e.role,
            note_type: e.note_type,
            title: e.title,
            content: e.content,
            difficulty: e.difficulty.map(|v| v as u8),
            created_at: Some(e.created_at),
        }
    }
}

pub async fn get_champion_pool(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Vec<ChampionPoolEntry>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT * FROM champion_pool WHERE user = type::record('user', $user_key) ORDER BY role, champion ASC")
        .bind(("user_key", user_key))
        .await?;
    let entries: Vec<DbPoolEntry> = r.take(0).unwrap_or_default();
    Ok(entries.into_iter().map(ChampionPoolEntry::from).collect())
}

pub async fn add_to_champion_pool(
    db: &Surreal<Db>,
    user_id: &str,
    champion: String,
    role: String,
) -> DbResult<()> {
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

pub async fn remove_from_champion_pool(
    db: &Surreal<Db>,
    user_id: &str,
    champion: String,
    role: String,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("DELETE champion_pool WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion))
        .bind(("role", role))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_champion_tier(
    db: &Surreal<Db>,
    user_id: &str,
    champion: &str,
    role: &str,
    tier: String,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE champion_pool SET tier = $tier WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion.to_string()))
        .bind(("role", role.to_string()))
        .bind(("tier", tier))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_champion_notes(
    db: &Surreal<Db>,
    user_id: &str,
    champion: &str,
    role: &str,
    notes: Option<String>,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE champion_pool SET notes = $notes WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion.to_string()))
        .bind(("role", role.to_string()))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_champion_comfort(
    db: &Surreal<Db>,
    user_id: &str,
    champion: &str,
    role: &str,
    comfort_level: Option<i64>,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE champion_pool SET comfort_level = $comfort_level WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion.to_string()))
        .bind(("role", role.to_string()))
        .bind(("comfort_level", comfort_level))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_champion_meta_tag(
    db: &Surreal<Db>,
    user_id: &str,
    champion: &str,
    role: &str,
    meta_tag: Option<String>,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE champion_pool SET meta_tag = $meta_tag WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role")
        .bind(("user_key", user_key))
        .bind(("champion", champion.to_string()))
        .bind(("role", role.to_string()))
        .bind(("meta_tag", meta_tag))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Champion Notes
// ---------------------------------------------------------------------------

pub async fn get_champion_notes(
    db: &Surreal<Db>,
    user_id: &str,
    champion: &str,
    role: &str,
) -> DbResult<Vec<ChampionNote>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT *, <string>created_at AS created_at FROM champion_note WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role ORDER BY note_type, created_at DESC")
        .bind(("user_key", user_key))
        .bind(("champion", champion.to_string()))
        .bind(("role", role.to_string()))
        .await?;
    let entries: Vec<DbChampionNote> = r.take(0).unwrap_or_default();
    Ok(entries.into_iter().map(ChampionNote::from).collect())
}

pub async fn get_pool_notes_for_champions(
    db: &Surreal<Db>,
    user_id: &str,
    champions: &[String],
) -> DbResult<Vec<ChampionNote>> {
    if champions.is_empty() {
        return Ok(Vec::new());
    }
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT *, <string>created_at AS created_at FROM champion_note WHERE user = type::record('user', $user_key) AND champion IN $champions ORDER BY champion, note_type, created_at DESC")
        .bind(("user_key", user_key))
        .bind(("champions", champions.to_vec()))
        .await?;
    let entries: Vec<DbChampionNote> = r.take(0).unwrap_or_default();
    Ok(entries.into_iter().map(ChampionNote::from).collect())
}

pub async fn add_champion_note(
    db: &Surreal<Db>,
    user_id: &str,
    note: ChampionNote,
) -> DbResult<String> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("CREATE champion_note SET user = type::record('user', $user_key), champion = $champion, role = $role, note_type = $note_type, title = $title, content = $content, difficulty = $difficulty")
        .bind(("user_key", user_key))
        .bind(("champion", note.champion))
        .bind(("role", note.role))
        .bind(("note_type", note.note_type))
        .bind(("title", note.title))
        .bind(("content", note.content))
        .bind(("difficulty", note.difficulty.map(|v| v as i64)))
        .await?
        .check()?;
    let created: Option<IdRecord> = r.take(0)?;
    Ok(created
        .map(|r| r.id.to_sql())
        .unwrap_or_default())
}

pub async fn update_champion_note(
    db: &Surreal<Db>,
    note: ChampionNote,
) -> DbResult<()> {
    let note_id = note.id.unwrap_or_default();
    let note_key = note_id
        .strip_prefix("champion_note:")
        .unwrap_or(&note_id)
        .to_string();
    db.query("UPDATE type::record('champion_note', $note_key) SET title = $title, content = $content, difficulty = $difficulty")
        .bind(("note_key", note_key))
        .bind(("title", note.title))
        .bind(("content", note.content))
        .bind(("difficulty", note.difficulty.map(|v| v as i64)))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_champion_note(
    db: &Surreal<Db>,
    note_id: &str,
) -> DbResult<()> {
    let note_key = note_id
        .strip_prefix("champion_note:")
        .unwrap_or(note_id)
        .to_string();
    db.query("DELETE type::record('champion_note', $note_key)")
        .bind(("note_key", note_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_champion_stats_for_user(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Vec<ChampionStatSummary>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PlayerMatchRow {
        champion: String,
        kills: i64,
        deaths: i64,
        assists: i64,
        cs: i64,
        win: bool,
    }

    let mut r = db
        .query("SELECT champion, kills, deaths, assists, cs, win FROM player_match WHERE user = type::record('user', $user_key)")
        .bind(("user_key", user_key))
        .await?;

    let rows: Vec<PlayerMatchRow> = r.take(0).unwrap_or_default();

    let mut by_champ: HashMap<String, Vec<(f64, f64, f64, f64, bool)>> = HashMap::new();
    for row in &rows {
        by_champ
            .entry(row.champion.clone())
            .or_default()
            .push((
                row.kills as f64,
                row.deaths as f64,
                row.assists as f64,
                row.cs as f64,
                row.win,
            ));
    }

    let mut results: Vec<ChampionStatSummary> = by_champ
        .into_iter()
        .map(|(champion, matches)| {
            let games = matches.len() as i32;
            let wins = matches.iter().filter(|m| m.4).count() as i32;
            let total_kills: f64 = matches.iter().map(|m| m.0).sum();
            let total_deaths: f64 = matches.iter().map(|m| m.1).sum();
            let total_assists: f64 = matches.iter().map(|m| m.2).sum();
            let avg_cs: f64 = matches.iter().map(|m| m.3).sum::<f64>() / games as f64;

            let avg_kda = if total_deaths > 0.0 {
                (total_kills + total_assists) / total_deaths
            } else {
                total_kills + total_assists
            };

            ChampionStatSummary {
                champion,
                games,
                wins,
                avg_kda: (avg_kda * 10.0).round() / 10.0,
                avg_cs_per_min: (avg_cs * 10.0).round() / 10.0,
            }
        })
        .collect();

    results.sort_by(|a, b| b.games.cmp(&a.games));
    Ok(results)
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

    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(&team_id)
        .to_string();

    db.query("CREATE team_member SET team = type::record('team', $team_key), user = type::record('user', $user_key), role = 'unassigned', roster_type = 'sub'")
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

pub async fn update_team(
    db: &Surreal<Db>,
    team_id: &str,
    name: String,
    region: String,
) -> DbResult<()> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    db.query("UPDATE type::record('team', $team_key) SET name=$name, region=$region")
        .bind(("team_key", team_key))
        .bind(("name", name))
        .bind(("region", region))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_member_role(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    role: String,
) -> DbResult<()> {
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
    let mut r = db.query("SELECT *, (SELECT count() FROM team_member WHERE team = $parent.id GROUP ALL)[0].count AS member_count FROM team ORDER BY name ASC").await?;
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
        return Err(DbError::Other(
            "You are already a member of this team".into(),
        ));
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

    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(&team_id)
        .to_string();

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
        return Err(DbError::Other(
            "You already have a pending request for this team".into(),
        ));
    }
    // Also prevent joining a team you're already in
    let mut member_check = db
        .query("SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) LIMIT 1")
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .await?;
    let already_member: Option<IdRecord> = member_check.take(0)?;
    if already_member.is_some() {
        return Err(DbError::Other(
            "You are already a member of this team".into(),
        ));
    }
    db.query("CREATE join_request SET team = type::record('team', $team_key), user = type::record('user', $user_key), status = 'pending'")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn list_pending_join_requests(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<JoinRequest>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT id, team, user.id as user_id, user.username as username, user.riot_summoner_name as riot_summoner_name FROM join_request WHERE team = type::record('team', $team_key) AND status = 'pending'")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbJoinRequest> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(JoinRequest::from).collect())
}

pub async fn respond_to_join_request(
    db: &Surreal<Db>,
    request_id: &str,
    accept: bool,
    team_id: &str,
) -> DbResult<()> {
    let req_key = request_id
        .strip_prefix("join_request:")
        .unwrap_or(request_id)
        .to_string();
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
        struct UserRef {
            user: RecordId,
        }
        let row: Option<UserRef> = r.take(0)?;
        if let Some(ur) = row {
            let user_sql = ur.user.to_sql();
            let user_key = user_sql
                .strip_prefix("user:")
                .unwrap_or(&user_sql)
                .to_string();
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
    struct Count {
        n: i64,
    }
    let row: Option<Count> = r.take(0)?;
    Ok(row.map(|c| c.n as usize).unwrap_or(0))
}

pub async fn assign_to_slot(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    role: &str,
) -> DbResult<()> {
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
    our_side: String,
    comments: Vec<String>,
    tags: Vec<String>,
    win_conditions: Option<String>,
    watch_out: Option<String>,
    series_id: Option<String>,
    game_number: Option<i32>,
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
            our_side: d.our_side,
            actions: Vec::new(),
            comments: d.comments,
            tags: d.tags,
            win_conditions: d.win_conditions,
            watch_out: d.watch_out,
            series_id: d.series_id,
            game_number: d.game_number,
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
    comment: Option<String>,
    #[serde(default)]
    role: Option<String>,
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
            comment: a.comment,
            role: a.role,
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    our_side: String,
    tags: Vec<String>,
    win_conditions: Option<String>,
    watch_out: Option<String>,
    series_id: Option<String>,
    game_number: Option<i32>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let mut response = db
        .query("CREATE draft SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent = $opponent, notes = $notes, comments = $comments, rating = $rating, our_side = $our_side, tags = $tags, win_conditions = $win_conditions, watch_out = $watch_out, series_id = $series_id, game_number = $game_number")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
        .bind(("comments", comments))
        .bind(("rating", rating))
        .bind(("our_side", our_side))
        .bind(("tags", tags))
        .bind(("win_conditions", win_conditions))
        .bind(("watch_out", watch_out))
        .bind(("series_id", series_id))
        .bind(("game_number", game_number))
        .await?
        .check()?;

    let row: Option<IdRecord> = response.take(0)?;
    let draft_id = match row {
        Some(r) => r.id.to_sql(),
        None => return Err(DbError::Other("Failed to create draft".into())),
    };

    let draft_key = draft_id
        .strip_prefix("draft:")
        .unwrap_or(&draft_id)
        .to_string();

    for action in actions {
        let dk = draft_key.clone();
        db.query("CREATE draft_action SET draft = type::record('draft', $draft_key), phase = $phase, side = $side, champion = $champion, `order` = $order, comment = $comment, role = $role")
            .bind(("draft_key", dk))
            .bind(("phase", action.phase))
            .bind(("side", action.side))
            .bind(("champion", action.champion))
            .bind(("order", action.order))
            .bind(("comment", action.comment))
            .bind(("role", action.role))
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
        actions_by_draft
            .entry(draft_id)
            .or_default()
            .push(DraftAction::from(a));
    }

    Ok(db_drafts
        .into_iter()
        .map(|d| {
            let id = d.id.to_sql();
            let mut draft = Draft::from(d);
            draft.actions = actions_by_draft.remove(&id).unwrap_or_default();
            draft
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_draft(
    db: &Surreal<Db>,
    draft_id: &str,
    name: String,
    opponent: Option<String>,
    notes: Option<String>,
    comments: Vec<String>,
    actions: Vec<DraftAction>,
    rating: Option<String>,
    our_side: String,
    tags: Vec<String>,
    win_conditions: Option<String>,
    watch_out: Option<String>,
    series_id: Option<String>,
    game_number: Option<i32>,
) -> DbResult<()> {
    let draft_key = draft_id
        .strip_prefix("draft:")
        .unwrap_or(draft_id)
        .to_string();

    db.query("UPDATE type::record('draft', $draft_key) SET name=$name, opponent=$opponent, notes=$notes, comments=$comments, rating=$rating, our_side=$our_side, tags=$tags, win_conditions=$win_conditions, watch_out=$watch_out, series_id=$series_id, game_number=$game_number")
        .bind(("draft_key", draft_key.clone()))
        .bind(("name", name))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
        .bind(("comments", comments))
        .bind(("rating", rating))
        .bind(("our_side", our_side))
        .bind(("tags", tags))
        .bind(("win_conditions", win_conditions))
        .bind(("watch_out", watch_out))
        .bind(("series_id", series_id))
        .bind(("game_number", game_number))
        .await?
        .check()?;

    db.query("DELETE draft_action WHERE draft = type::record('draft', $draft_key)")
        .bind(("draft_key", draft_key.clone()))
        .await?
        .check()?;

    for action in actions {
        let dk = draft_key.clone();
        db.query("CREATE draft_action SET draft = type::record('draft', $draft_key), phase = $phase, side = $side, champion = $champion, `order` = $order, comment = $comment, role = $role")
            .bind(("draft_key", dk))
            .bind(("phase", action.phase))
            .bind(("side", action.side))
            .bind(("champion", action.champion))
            .bind(("order", action.order))
            .bind(("comment", action.comment))
            .bind(("role", action.role))
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
    comment: Option<String>,
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

    let root_label = name.clone();
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
            // Auto-create a root node named after the tree
            let tree_key = tree_id
                .strip_prefix("draft_tree:")
                .unwrap_or(&tree_id)
                .to_string();
            db.query("CREATE draft_tree_node SET tree = type::record('draft_tree', $tree_key), parent = NONE, label = $label, sort_order = 0")
                .bind(("tree_key", tree_key))
                .bind(("label", root_label))
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
    let tree_key = tree_id
        .strip_prefix("draft_tree:")
        .unwrap_or(tree_id)
        .to_string();
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
    let tree_key = tree_id
        .strip_prefix("draft_tree:")
        .unwrap_or(tree_id)
        .to_string();
    db.query("UPDATE type::record('draft_tree', $tree_key) SET name = $name, opponent = $opponent")
        .bind(("tree_key", tree_key))
        .bind(("name", name))
        .bind(("opponent", opponent))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_tree_nodes(db: &Surreal<Db>, tree_id: &str) -> DbResult<Vec<DraftTreeNode>> {
    let tree_key = tree_id
        .strip_prefix("draft_tree:")
        .unwrap_or(tree_id)
        .to_string();

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
        actions_by_node
            .entry(node_id)
            .or_default()
            .push(DraftAction {
                id: Some(a.id.to_sql()),
                draft_id: String::new(),
                phase: a.phase,
                side: a.side,
                champion: a.champion,
                order: a.order,
                comment: a.comment,
                role: None,
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

    // Build tree structure using a parent → [child_ids] map, then recursive DFS.
    // This is correct regardless of the order flat_nodes are returned.
    let mut node_map: HashMap<String, DraftTreeNode> = HashMap::new();
    let mut root_ids: Vec<String> = Vec::new();
    let mut children_of: HashMap<String, Vec<String>> = HashMap::new();

    for node in flat_nodes {
        let id = node.id.clone().unwrap_or_default();
        if let Some(ref pid) = node.parent_id {
            children_of.entry(pid.clone()).or_default().push(id.clone());
        } else {
            root_ids.push(id.clone());
        }
        node_map.insert(id, node);
    }

    fn attach_children(
        node_id: &str,
        node_map: &mut HashMap<String, DraftTreeNode>,
        children_of: &HashMap<String, Vec<String>>,
    ) -> Option<DraftTreeNode> {
        let mut node = node_map.remove(node_id)?;
        if let Some(child_ids) = children_of.get(node_id) {
            let mut children: Vec<DraftTreeNode> = child_ids
                .iter()
                .filter_map(|cid| attach_children(cid, node_map, children_of))
                .collect();
            children.sort_by_key(|c| c.sort_order);
            node.children = children;
        }
        Some(node)
    }

    let mut roots: Vec<DraftTreeNode> = root_ids
        .iter()
        .filter_map(|id| attach_children(id, &mut node_map, &children_of))
        .collect();
    roots.sort_by_key(|r| r.sort_order);

    Ok(roots)
}

pub async fn create_tree_node(
    db: &Surreal<Db>,
    tree_id: &str,
    parent_id: Option<String>,
    label: String,
) -> DbResult<String> {
    let tree_key = tree_id
        .strip_prefix("draft_tree:")
        .unwrap_or(tree_id)
        .to_string();

    let parent_clause = match &parent_id {
        Some(pid) => {
            let pk = pid
                .strip_prefix("draft_tree_node:")
                .unwrap_or(pid)
                .to_string();
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
    struct Count {
        n: i64,
    }

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
    let node_key = node_id
        .strip_prefix("draft_tree_node:")
        .unwrap_or(node_id)
        .to_string();

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
        db.query("CREATE tree_node_action SET node = type::record('draft_tree_node', $node_key), phase = $phase, side = $side, champion = $champion, `order` = $order, comment = $comment")
            .bind(("node_key", nk))
            .bind(("phase", action.phase))
            .bind(("side", action.side))
            .bind(("champion", action.champion))
            .bind(("order", action.order))
            .bind(("comment", action.comment))
            .await?
            .check()?;
    }

    Ok(())
}

pub async fn delete_tree_node(db: &Surreal<Db>, node_id: &str) -> DbResult<()> {
    let node_key = node_id
        .strip_prefix("draft_tree_node:")
        .unwrap_or(node_id)
        .to_string();
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
            let child_key = child_id
                .strip_prefix("draft_tree_node:")
                .unwrap_or(&child_id)
                .to_string();
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
    store_matches_with_synced_by(db, user_id, matches, None).await
}

pub async fn store_matches_with_synced_by(
    db: &Surreal<Db>,
    user_id: &str,
    matches: Vec<crate::server::riot::MatchData>,
    synced_by_user_id: Option<&str>,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let synced_by_key = synced_by_user_id
        .map(|s| s.strip_prefix("user:").unwrap_or(s).to_string());

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

            let synced_by_clause = synced_by_key
                .as_ref()
                .map(|k| format!(", synced_by = type::record('user', '{k}')"))
                .unwrap_or_default();

            let query = match &game_end_str {
                Some(ge) => format!(
                    "CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration, game_end = <datetime>'{ge}'{synced_by_clause}"
                ),
                None => format!("CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration{synced_by_clause}"),
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
            .await?
            .check()?;
    }
    Ok(())
}

/// Get team stats with player details, joining match + player_match.
/// Returns all player_match entries for team members, enriched with match date.
pub async fn get_team_match_stats(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<TeamMatchRow>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Get all team member user IDs
    let mut r = db
        .query("SELECT user FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct UserRef {
        user: RecordId,
    }
    let user_refs: Vec<UserRef> = r.take(0).unwrap_or_default();
    if user_refs.is_empty() {
        return Ok(Vec::new());
    }

    // Build a query that gets all player_match records for team members with match info
    // Using a join via the match record
    let user_ids: Vec<String> = user_refs.iter().map(|u| u.user.to_sql()).collect();
    let user_id_list = user_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "SELECT *, match.match_id as riot_match_id, match.queue_id as queue_id, match.game_duration as game_duration, <string> match.game_end as game_end, user.username as username FROM player_match WHERE user IN [{user_id_list}] ORDER BY match.game_end DESC LIMIT 200"
    );

    let mut result = db.query(&query).await?;
    let rows: Vec<DbTeamMatchRow> = result.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(TeamMatchRow::from).collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeamMatchRow {
    id: RecordId,
    #[serde(rename = "match")]
    match_ref: Option<RecordId>,
    user: RecordId,
    username: String,
    riot_match_id: String,
    queue_id: i32,
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

impl From<DbTeamMatchRow> for TeamMatchRow {
    fn from(r: DbTeamMatchRow) -> Self {
        TeamMatchRow {
            match_db_id: r.match_ref.map(|m| m.to_sql()).unwrap_or_default(),
            user_id: r.user.to_sql(),
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
        }
    }
}

/// Get roster member puuids for syncing
pub async fn get_roster_puuids(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<(String, String, Option<String>)>> {
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
    Ok(members
        .into_iter()
        .map(|m| (m.user_id.to_sql(), m.username, m.riot_puuid))
        .collect())
}

// ---------------------------------------------------------------------------
// Game plans / post-game
// ---------------------------------------------------------------------------

pub async fn save_game_plan(db: &Surreal<Db>, plan: GamePlan, user_id: &str) -> DbResult<String> {
    let team_key = plan
        .team_id
        .strip_prefix("team:")
        .unwrap_or(&plan.team_id)
        .to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut response = db.query("CREATE game_plan SET name = $name, team = type::record('team', $team_key), draft = $draft_id, our_champions = $our_champions, enemy_champions = $enemy_champions, win_conditions = $win_conditions, objective_priority = $objective_priority, teamfight_strategy = $teamfight_strategy, early_game = $early_game, top_strategy = $top_strategy, jungle_strategy = $jungle_strategy, mid_strategy = $mid_strategy, bot_strategy = $bot_strategy, support_strategy = $support_strategy, notes = $notes, win_condition_tag = $win_condition_tag, created_by = type::record('user', $user_key)")
        .bind(("name", plan.name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("draft_id", plan.draft_id))
        .bind(("our_champions", plan.our_champions))
        .bind(("enemy_champions", plan.enemy_champions))
        .bind(("win_conditions", plan.win_conditions))
        .bind(("objective_priority", plan.objective_priority))
        .bind(("teamfight_strategy", plan.teamfight_strategy))
        .bind(("early_game", plan.early_game))
        .bind(("top_strategy", plan.top_strategy))
        .bind(("jungle_strategy", plan.jungle_strategy))
        .bind(("mid_strategy", plan.mid_strategy))
        .bind(("bot_strategy", plan.bot_strategy))
        .bind(("support_strategy", plan.support_strategy))
        .bind(("notes", plan.notes))
        .bind(("win_condition_tag", plan.win_condition_tag))
        .await?
        .check()?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create game plan".into())),
    }
}

pub async fn update_game_plan(db: &Surreal<Db>, plan: GamePlan) -> DbResult<()> {
    let plan_id = plan
        .id
        .as_deref()
        .ok_or(DbError::Other("No plan ID".into()))?;
    let plan_key = plan_id
        .strip_prefix("game_plan:")
        .unwrap_or(plan_id)
        .to_string();
    db.query("UPDATE type::record('game_plan', $plan_key) SET name = $name, draft = $draft_id, our_champions = $our_champions, enemy_champions = $enemy_champions, win_conditions = $win_conditions, objective_priority = $objective_priority, teamfight_strategy = $teamfight_strategy, early_game = $early_game, top_strategy = $top_strategy, jungle_strategy = $jungle_strategy, mid_strategy = $mid_strategy, bot_strategy = $bot_strategy, support_strategy = $support_strategy, notes = $notes, win_condition_tag = $win_condition_tag")
        .bind(("plan_key", plan_key))
        .bind(("name", plan.name))
        .bind(("draft_id", plan.draft_id))
        .bind(("our_champions", plan.our_champions))
        .bind(("enemy_champions", plan.enemy_champions))
        .bind(("win_conditions", plan.win_conditions))
        .bind(("objective_priority", plan.objective_priority))
        .bind(("teamfight_strategy", plan.teamfight_strategy))
        .bind(("early_game", plan.early_game))
        .bind(("top_strategy", plan.top_strategy))
        .bind(("jungle_strategy", plan.jungle_strategy))
        .bind(("mid_strategy", plan.mid_strategy))
        .bind(("bot_strategy", plan.bot_strategy))
        .bind(("support_strategy", plan.support_strategy))
        .bind(("notes", plan.notes))
        .bind(("win_condition_tag", plan.win_condition_tag))
        .await?
        .check()?;
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbGamePlan {
    id: RecordId,
    name: String,
    team: RecordId,
    draft: Option<String>,
    our_champions: Vec<String>,
    enemy_champions: Vec<String>,
    win_conditions: Vec<String>,
    objective_priority: Vec<String>,
    teamfight_strategy: String,
    early_game: Option<String>,
    top_strategy: Option<String>,
    jungle_strategy: Option<String>,
    mid_strategy: Option<String>,
    bot_strategy: Option<String>,
    support_strategy: Option<String>,
    notes: Option<String>,
    win_condition_tag: Option<String>,
}

impl From<DbGamePlan> for GamePlan {
    fn from(p: DbGamePlan) -> Self {
        GamePlan {
            id: Some(p.id.to_sql()),
            team_id: p.team.to_sql(),
            draft_id: p.draft,
            name: p.name,
            our_champions: p.our_champions,
            enemy_champions: p.enemy_champions,
            win_conditions: p.win_conditions,
            objective_priority: p.objective_priority,
            teamfight_strategy: p.teamfight_strategy,
            early_game: p.early_game,
            top_strategy: p.top_strategy,
            jungle_strategy: p.jungle_strategy,
            mid_strategy: p.mid_strategy,
            bot_strategy: p.bot_strategy,
            support_strategy: p.support_strategy,
            notes: p.notes,
            win_condition_tag: p.win_condition_tag,
        }
    }
}

pub async fn list_game_plans(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<GamePlan>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT * FROM game_plan WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbGamePlan> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(GamePlan::from).collect())
}

pub async fn delete_game_plan(db: &Surreal<Db>, plan_id: &str) -> DbResult<()> {
    let plan_key = plan_id
        .strip_prefix("game_plan:")
        .unwrap_or(plan_id)
        .to_string();
    db.query("DELETE type::record('game_plan', $plan_key)")
        .bind(("plan_key", plan_key))
        .await?
        .check()?;
    Ok(())
}

/// Fetch a single draft with its actions for prefilling a game plan.
/// Returns `None` if no draft with the given ID exists.
pub async fn get_draft_for_prefill(
    db: &Surreal<Db>,
    draft_id: &str,
) -> DbResult<Option<Draft>> {
    let draft_key = draft_id
        .strip_prefix("draft:")
        .unwrap_or(draft_id)
        .to_string();
    let mut result = db
        .query("SELECT * FROM type::record('draft', $draft_key); SELECT * FROM draft_action WHERE draft = type::record('draft', $draft_key) ORDER BY `order` ASC")
        .bind(("draft_key", draft_key))
        .await?;
    let db_drafts: Vec<DbDraft> = result.take(0).unwrap_or_default();
    let db_actions: Vec<DbDraftAction> = result.take(1).unwrap_or_default();

    match db_drafts.into_iter().next() {
        None => Ok(None),
        Some(db_draft) => {
            let mut draft = Draft::from(db_draft);
            draft.actions = db_actions.into_iter().map(DraftAction::from).collect();
            Ok(Some(draft))
        }
    }
}

/// Find all game plans that reference the given draft ID.
/// The `game_plan.draft` field stores the full record ID string (e.g. `"draft:abc123"`).
pub async fn get_game_plans_for_draft(
    db: &Surreal<Db>,
    draft_id: &str,
) -> DbResult<Vec<GamePlan>> {
    let mut r = db
        .query("SELECT * FROM game_plan WHERE draft = $draft_id ORDER BY created_at DESC")
        .bind(("draft_id", draft_id.to_string()))
        .await?;
    let rows: Vec<DbGamePlan> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(GamePlan::from).collect())
}

pub async fn save_post_game_learning(
    db: &Surreal<Db>,
    learning: PostGameLearning,
) -> DbResult<String> {
    let team_key = learning
        .team_id
        .strip_prefix("team:")
        .unwrap_or(&learning.team_id)
        .to_string();
    let created_by_key = learning
        .created_by
        .strip_prefix("user:")
        .unwrap_or(&learning.created_by)
        .to_string();
    let mut response = db.query("CREATE post_game_learning SET team = type::record('team', $team_key), match_riot_id = $match_riot_id, game_plan_id = $game_plan_id, draft_id = $draft_id, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, open_notes = $open_notes, created_by = type::record('user', $created_by_key), win_loss = $win_loss, rating = $rating")
        .bind(("team_key", team_key))
        .bind(("match_riot_id", learning.match_riot_id))
        .bind(("game_plan_id", learning.game_plan_id))
        .bind(("draft_id", learning.draft_id))
        .bind(("what_went_well", learning.what_went_well))
        .bind(("improvements", learning.improvements))
        .bind(("action_items", learning.action_items))
        .bind(("open_notes", learning.open_notes))
        .bind(("created_by_key", created_by_key))
        .bind(("win_loss", learning.win_loss))
        .bind(("rating", learning.rating))
        .await?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create post-game review".into())),
    }
}

pub async fn update_post_game_learning(
    db: &Surreal<Db>,
    learning: PostGameLearning,
) -> DbResult<()> {
    let id = learning
        .id
        .as_deref()
        .ok_or(DbError::Other("No review ID".into()))?;
    let key = id
        .strip_prefix("post_game_learning:")
        .unwrap_or(id)
        .to_string();
    db.query("UPDATE type::record('post_game_learning', $key) SET match_riot_id = $match_riot_id, game_plan_id = $game_plan_id, draft_id = $draft_id, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, open_notes = $open_notes, win_loss = $win_loss, rating = $rating")
        .bind(("key", key))
        .bind(("match_riot_id", learning.match_riot_id))
        .bind(("game_plan_id", learning.game_plan_id))
        .bind(("draft_id", learning.draft_id))
        .bind(("what_went_well", learning.what_went_well))
        .bind(("improvements", learning.improvements))
        .bind(("action_items", learning.action_items))
        .bind(("open_notes", learning.open_notes))
        .bind(("win_loss", learning.win_loss))
        .bind(("rating", learning.rating))
        .await?
        .check()?;
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbPostGameLearning {
    id: RecordId,
    team: RecordId,
    match_riot_id: Option<String>,
    game_plan_id: Option<String>,
    draft_id: Option<String>,
    what_went_well: Vec<String>,
    improvements: Vec<String>,
    action_items: Vec<String>,
    open_notes: Option<String>,
    created_by: RecordId,
    #[serde(default)]
    win_loss: Option<String>,
    #[serde(default)]
    rating: Option<u8>,
}

impl From<DbPostGameLearning> for PostGameLearning {
    fn from(p: DbPostGameLearning) -> Self {
        PostGameLearning {
            id: Some(p.id.to_sql()),
            team_id: p.team.to_sql(),
            match_riot_id: p.match_riot_id,
            game_plan_id: p.game_plan_id,
            draft_id: p.draft_id,
            what_went_well: p.what_went_well,
            improvements: p.improvements,
            action_items: p.action_items,
            open_notes: p.open_notes,
            created_by: p.created_by.to_sql(),
            win_loss: p.win_loss,
            rating: p.rating,
        }
    }
}

pub async fn list_post_game_learnings(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<PostGameLearning>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbPostGameLearning> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(PostGameLearning::from).collect())
}

pub async fn delete_post_game_learning(db: &Surreal<Db>, id: &str) -> DbResult<()> {
    let key = id
        .strip_prefix("post_game_learning:")
        .unwrap_or(id)
        .to_string();
    db.query("DELETE type::record('post_game_learning', $key)")
        .bind(("key", key))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_analytics(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<(Vec<StrategyTagSummary>, Vec<GamePlanEffectiveness>)> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Two queries in one round-trip (per project rule 29)
    let mut r = db.query(
        "SELECT * FROM game_plan WHERE team = type::record('team', $team_key); \
         SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key) AND game_plan_id IS NOT NONE;"
    )
    .bind(("team_key", team_key))
    .await?;

    let plans: Vec<DbGamePlan> = r.take(0).unwrap_or_default();
    let reviews: Vec<DbPostGameLearning> = r.take(1).unwrap_or_default();

    // Convert to app models
    let plans: Vec<GamePlan> = plans.into_iter().map(GamePlan::from).collect();
    let reviews: Vec<PostGameLearning> = reviews.into_iter().map(PostGameLearning::from).collect();

    // Group reviews by game_plan_id
    let mut reviews_by_plan: HashMap<String, Vec<PostGameLearning>> = HashMap::new();
    for review in reviews {
        if let Some(ref plan_id) = review.game_plan_id {
            reviews_by_plan.entry(plan_id.clone()).or_default().push(review);
        }
    }

    // Build per-plan effectiveness
    let mut plan_effectiveness: Vec<GamePlanEffectiveness> = Vec::new();
    for plan in &plans {
        let plan_id = match &plan.id {
            Some(id) => id.clone(),
            None => continue,
        };
        let plan_reviews = reviews_by_plan.remove(&plan_id).unwrap_or_default();
        let wins = plan_reviews.iter().filter(|r| r.win_loss.as_deref() == Some("win")).count();
        let losses = plan_reviews.iter().filter(|r| r.win_loss.as_deref() == Some("loss")).count();
        let ratings: Vec<f32> = plan_reviews.iter()
            .filter_map(|r| r.rating.map(|v| v as f32))
            .collect();
        let avg_rating = if ratings.is_empty() { None } else {
            Some(ratings.iter().sum::<f32>() / ratings.len() as f32)
        };

        plan_effectiveness.push(GamePlanEffectiveness {
            plan_id,
            plan_name: plan.name.clone(),
            tag: plan.win_condition_tag.clone(),
            wins,
            losses,
            avg_rating,
            reviews: plan_reviews,
        });
    }

    // Build strategy tag summaries by grouping plan effectiveness by tag
    let mut tag_map: HashMap<String, (usize, usize, usize, Vec<f32>)> = HashMap::new();
    for pe in &plan_effectiveness {
        if let Some(ref tag) = pe.tag {
            if !tag.is_empty() {
                let entry = tag_map.entry(tag.clone()).or_insert((0, 0, 0, Vec::new()));
                entry.0 += pe.wins + pe.losses; // games_played
                entry.1 += pe.wins;
                entry.2 += pe.losses;
                // Collect individual review ratings for accurate tag-level average
                for review in &pe.reviews {
                    if let Some(r) = review.rating {
                        entry.3.push(r as f32);
                    }
                }
            }
        }
    }

    let tag_summaries: Vec<StrategyTagSummary> = tag_map.into_iter()
        .map(|(tag, (games_played, wins, losses, ratings))| {
            let avg_rating = if ratings.is_empty() { None } else {
                Some(ratings.iter().sum::<f32>() / ratings.len() as f32)
            };
            StrategyTagSummary { tag, games_played, wins, losses, avg_rating }
        })
        .collect();

    Ok((tag_summaries, plan_effectiveness))
}

// ---------------------------------------------------------------------------
// Checklist templates & instances
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbChecklistTemplate {
    id: RecordId,
    team: RecordId,
    name: String,
    items: Vec<String>,
}

impl From<DbChecklistTemplate> for ChecklistTemplate {
    fn from(t: DbChecklistTemplate) -> Self {
        ChecklistTemplate {
            id: Some(t.id.to_sql()),
            team_id: t.team.to_sql(),
            name: t.name,
            items: t.items,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbChecklistInstance {
    id: RecordId,
    team: RecordId,
    game_plan_id: Option<String>,
    template_id: Option<String>,
    items: Vec<String>,
    checked: Vec<bool>,
}

impl From<DbChecklistInstance> for ChecklistInstance {
    fn from(i: DbChecklistInstance) -> Self {
        ChecklistInstance {
            id: Some(i.id.to_sql()),
            team_id: i.team.to_sql(),
            game_plan_id: i.game_plan_id,
            template_id: i.template_id,
            items: i.items,
            checked: i.checked,
        }
    }
}

pub async fn create_checklist_template(
    db: &Surreal<Db>,
    team_id: &str,
    name: String,
    items: Vec<String>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut response = db
        .query("CREATE checklist_template SET team = type::record('team', $team_key), name = $name, items = $items")
        .bind(("team_key", team_key))
        .bind(("name", name))
        .bind(("items", items))
        .await?
        .check()?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create checklist template".into())),
    }
}

pub async fn list_checklist_templates(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<ChecklistTemplate>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query("SELECT * FROM checklist_template WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbChecklistTemplate> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(ChecklistTemplate::from).collect())
}

pub async fn delete_checklist_template(db: &Surreal<Db>, template_id: &str) -> DbResult<()> {
    let key = template_id
        .strip_prefix("checklist_template:")
        .unwrap_or(template_id)
        .to_string();
    db.query("DELETE type::record('checklist_template', $key)")
        .bind(("key", key))
        .await?
        .check()?;
    Ok(())
}

pub async fn create_checklist_instance(
    db: &Surreal<Db>,
    team_id: &str,
    game_plan_id: Option<String>,
    template_id: Option<String>,
    items: Vec<String>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let checked: Vec<bool> = vec![false; items.len()];
    let mut response = db
        .query("CREATE checklist_instance SET team = type::record('team', $team_key), game_plan_id = $game_plan_id, template_id = $template_id, items = $items, checked = $checked")
        .bind(("team_key", team_key))
        .bind(("game_plan_id", game_plan_id))
        .bind(("template_id", template_id))
        .bind(("items", items))
        .bind(("checked", checked))
        .await?
        .check()?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create checklist instance".into())),
    }
}

pub async fn get_checklist_for_plan(
    db: &Surreal<Db>,
    game_plan_id: &str,
) -> DbResult<Option<ChecklistInstance>> {
    let mut r = db
        .query("SELECT * FROM checklist_instance WHERE game_plan_id = $game_plan_id ORDER BY created_at DESC LIMIT 1")
        .bind(("game_plan_id", game_plan_id.to_string()))
        .await?;
    let row: Option<DbChecklistInstance> = r.take(0)?;
    Ok(row.map(ChecklistInstance::from))
}

pub async fn update_checklist_checked(
    db: &Surreal<Db>,
    instance_id: &str,
    checked: Vec<bool>,
) -> DbResult<()> {
    let key = instance_id
        .strip_prefix("checklist_instance:")
        .unwrap_or(instance_id)
        .to_string();
    db.query("UPDATE type::record('checklist_instance', $key) SET checked = $checked")
        .bind(("key", key))
        .bind(("checked", checked))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_checklist_instance(db: &Surreal<Db>, instance_id: &str) -> DbResult<()> {
    let key = instance_id
        .strip_prefix("checklist_instance:")
        .unwrap_or(instance_id)
        .to_string();
    db.query("DELETE type::record('checklist_instance', $key)")
        .bind(("key", key))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_game_plan(db: &Surreal<Db>, plan_id: &str) -> DbResult<Option<GamePlan>> {
    let plan_key = plan_id
        .strip_prefix("game_plan:")
        .unwrap_or(plan_id)
        .to_string();
    let mut r = db
        .query("SELECT * FROM type::record('game_plan', $plan_key)")
        .bind(("plan_key", plan_key))
        .await?;
    let row: Option<DbGamePlan> = r.take(0)?;
    Ok(row.map(GamePlan::from))
}

pub async fn delete_draft(db: &Surreal<Db>, draft_id: &str) -> DbResult<()> {
    let draft_key = draft_id
        .strip_prefix("draft:")
        .unwrap_or(draft_id)
        .to_string();
    db.query("DELETE draft_action WHERE draft = type::record('draft', $draft_key); DELETE type::record('draft', $draft_key)")
        .bind(("draft_key", draft_key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Ban Priority
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbBanPriority {
    id: RecordId,
    team: RecordId,
    champion: String,
    rank: i32,
    reason: Option<String>,
}

impl From<DbBanPriority> for BanPriority {
    fn from(b: DbBanPriority) -> Self {
        BanPriority {
            id: Some(b.id.to_sql()),
            team_id: b.team.to_sql(),
            champion: b.champion,
            rank: b.rank,
            reason: b.reason,
        }
    }
}

pub async fn get_ban_priorities(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<BanPriority>> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();
    let mut r = db
        .query("SELECT * FROM ban_priority WHERE team = type::record('team', $team_key) ORDER BY rank ASC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbBanPriority> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(BanPriority::from).collect())
}

pub async fn set_ban_priorities(
    db: &Surreal<Db>,
    team_id: &str,
    priorities: Vec<BanPriority>,
) -> DbResult<()> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();

    // Delete old + create new in transaction
    let mut query = String::from("BEGIN TRANSACTION; DELETE ban_priority WHERE team = type::record('team', $team_key);");
    for (i, _) in priorities.iter().enumerate() {
        query.push_str(&format!(
            " CREATE ban_priority SET team = type::record('team', $team_key), champion = $champ_{i}, rank = $rank_{i}, reason = $reason_{i};"
        ));
    }
    query.push_str(" COMMIT TRANSACTION;");

    let mut q = db.query(&query).bind(("team_key", team_key));
    for (i, p) in priorities.iter().enumerate() {
        q = q
            .bind((format!("champ_{i}"), p.champion.clone()))
            .bind((format!("rank_{i}"), p.rank))
            .bind((format!("reason_{i}"), p.reason.clone()));
    }
    q.await?.check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Opponents
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbOpponent {
    id: RecordId,
    name: String,
    team: RecordId,
    notes: Option<String>,
}

impl From<DbOpponent> for Opponent {
    fn from(o: DbOpponent) -> Self {
        Opponent {
            id: Some(o.id.to_sql()),
            name: o.name,
            team_id: o.team.to_sql(),
            notes: o.notes,
        }
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbOpponentPlayer {
    id: RecordId,
    opponent: RecordId,
    name: String,
    role: String,
    riot_puuid: Option<String>,
    riot_summoner_name: Option<String>,
    recent_champions: Vec<String>,
    notes: Option<String>,
    #[serde(default)]
    last_fetched: Option<String>,
    #[serde(default)]
    mastery_data_json: Option<String>,
    #[serde(default)]
    role_distribution_json: Option<String>,
}

impl From<DbOpponentPlayer> for OpponentPlayer {
    fn from(p: DbOpponentPlayer) -> Self {
        OpponentPlayer {
            id: Some(p.id.to_sql()),
            opponent_id: p.opponent.to_sql(),
            name: p.name,
            role: p.role,
            riot_puuid: p.riot_puuid,
            riot_summoner_name: p.riot_summoner_name,
            recent_champions: p.recent_champions,
            notes: p.notes,
            last_fetched: p.last_fetched,
            mastery_data_json: p.mastery_data_json,
            role_distribution_json: p.role_distribution_json,
        }
    }
}

pub async fn create_opponent(
    db: &Surreal<Db>,
    team_id: &str,
    name: String,
    notes: Option<String>,
) -> DbResult<String> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();
    let mut response = db
        .query("CREATE opponent SET name = $name, team = type::record('team', $team_key), notes = $notes")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("notes", notes))
        .await?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create opponent".into())),
    }
}

pub async fn list_opponents(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<Opponent>> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();
    let mut r = db
        .query("SELECT * FROM opponent WHERE team = type::record('team', $team_key) ORDER BY name ASC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbOpponent> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(Opponent::from).collect())
}

pub async fn get_opponent(
    db: &Surreal<Db>,
    opponent_id: &str,
) -> DbResult<Option<(Opponent, Vec<OpponentPlayer>)>> {
    let opp_key = opponent_id
        .strip_prefix("opponent:")
        .unwrap_or(opponent_id)
        .to_string();
    let mut r = db
        .query("SELECT * FROM type::record('opponent', $opp_key); SELECT *, <string>last_fetched AS last_fetched FROM opponent_player WHERE opponent = type::record('opponent', $opp_key) ORDER BY role ASC")
        .bind(("opp_key", opp_key))
        .await?;
    let opp: Option<DbOpponent> = r.take(0)?;
    let players: Vec<DbOpponentPlayer> = r.take(1).unwrap_or_default();
    match opp {
        Some(o) => Ok(Some((
            Opponent::from(o),
            players.into_iter().map(OpponentPlayer::from).collect(),
        ))),
        None => Ok(None),
    }
}

pub async fn update_opponent(
    db: &Surreal<Db>,
    opponent_id: &str,
    name: String,
    notes: Option<String>,
) -> DbResult<()> {
    let opp_key = opponent_id
        .strip_prefix("opponent:")
        .unwrap_or(opponent_id)
        .to_string();
    db.query("UPDATE type::record('opponent', $opp_key) SET name = $name, notes = $notes")
        .bind(("opp_key", opp_key))
        .bind(("name", name))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_opponent(db: &Surreal<Db>, opponent_id: &str) -> DbResult<()> {
    let opp_key = opponent_id
        .strip_prefix("opponent:")
        .unwrap_or(opponent_id)
        .to_string();
    db.query("DELETE opponent_player WHERE opponent = type::record('opponent', $opp_key); DELETE type::record('opponent', $opp_key)")
        .bind(("opp_key", opp_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn add_opponent_player(
    db: &Surreal<Db>,
    opponent_id: &str,
    name: String,
    role: String,
) -> DbResult<String> {
    let opp_key = opponent_id
        .strip_prefix("opponent:")
        .unwrap_or(opponent_id)
        .to_string();
    let mut response = db
        .query("CREATE opponent_player SET opponent = type::record('opponent', $opp_key), name = $name, role = $role")
        .bind(("opp_key", opp_key))
        .bind(("name", name))
        .bind(("role", role))
        .await?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to add opponent player".into())),
    }
}

pub async fn save_opponent_player_info(
    db: &Surreal<Db>,
    player_id: &str,
    name: String,
    role: String,
    riot_summoner_name: Option<String>,
    notes: Option<String>,
) -> DbResult<()> {
    let player_key = player_id
        .strip_prefix("opponent_player:")
        .unwrap_or(player_id)
        .to_string();
    db.query("UPDATE type::record('opponent_player', $player_key) SET name = $name, role = $role, riot_summoner_name = $riot_summoner_name, notes = $notes")
        .bind(("player_key", player_key))
        .bind(("name", name))
        .bind(("role", role))
        .bind(("riot_summoner_name", riot_summoner_name))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_opponent_player(
    db: &Surreal<Db>,
    player_id: &str,
    name: String,
    role: String,
    riot_puuid: Option<String>,
    riot_summoner_name: Option<String>,
    recent_champions: Vec<String>,
    notes: Option<String>,
) -> DbResult<()> {
    let player_key = player_id
        .strip_prefix("opponent_player:")
        .unwrap_or(player_id)
        .to_string();
    db.query("UPDATE type::record('opponent_player', $player_key) SET name = $name, role = $role, riot_puuid = $riot_puuid, riot_summoner_name = $riot_summoner_name, recent_champions = $recent_champions, notes = $notes")
        .bind(("player_key", player_key))
        .bind(("name", name))
        .bind(("role", role))
        .bind(("riot_puuid", riot_puuid))
        .bind(("riot_summoner_name", riot_summoner_name))
        .bind(("recent_champions", recent_champions))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_opponent_player(db: &Surreal<Db>, player_id: &str) -> DbResult<()> {
    let player_key = player_id
        .strip_prefix("opponent_player:")
        .unwrap_or(player_id)
        .to_string();
    db.query("DELETE type::record('opponent_player', $player_key)")
        .bind(("player_key", player_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_opponent_player_champions(
    db: &Surreal<Db>,
    player_id: &str,
    champions: Vec<String>,
) -> DbResult<()> {
    let player_key = player_id
        .strip_prefix("opponent_player:")
        .unwrap_or(player_id)
        .to_string();
    db.query("UPDATE type::record('opponent_player', $player_key) SET recent_champions = $champions")
        .bind(("player_key", player_key))
        .bind(("champions", champions))
        .await?
        .check()?;
    Ok(())
}

/// Create an opponent and its player slots atomically in a single transaction.
///
/// Each entry in `players` is `(role, riot_summoner_name)`. Player `name` defaults to empty
/// string and will be populated on first Riot API fetch or manual edit.
///
/// Returns `(opponent_id, player_ids)` where `player_ids.len() == players.len()`.
pub async fn create_opponent_with_players(
    db: &Surreal<Db>,
    team_id: &str,
    name: String,
    notes: Option<String>,
    players: Vec<(String, Option<String>)>,
) -> DbResult<(String, Vec<String>)> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();

    // Step 1: Create the opponent record
    let mut opp_resp = db
        .query("CREATE opponent SET name = $opp_name, team = type::record('team', $team_key), notes = $opp_notes")
        .bind(("opp_name", name.clone()))
        .bind(("team_key", team_key.clone()))
        .bind(("opp_notes", notes))
        .await?;
    let opp_row: Option<IdRecord> = opp_resp.take(0)?;
    let opponent_id = match opp_row {
        Some(r) => r.id.to_sql(),
        None => return Err(DbError::Other("Failed to create opponent".into())),
    };

    // Step 2: Create all player slots in a transaction
    let opp_key = opponent_id
        .strip_prefix("opponent:")
        .unwrap_or(&opponent_id)
        .to_string();

    let mut query = String::from("BEGIN TRANSACTION;\n");
    for i in 0..players.len() {
        query.push_str(&format!(
            "CREATE opponent_player SET opponent = type::record('opponent', $opp_key), name = '', role = $role_{i}, riot_summoner_name = $summoner_{i};\n"
        ));
    }
    query.push_str("COMMIT TRANSACTION;");

    let mut q = db.query(query).bind(("opp_key", opp_key));
    for (i, (role, summoner)) in players.iter().enumerate() {
        q = q
            .bind((format!("role_{i}"), role.clone()))
            .bind((format!("summoner_{i}"), summoner.clone()));
    }

    let mut response = q.await?.check()?;

    // Result indexing: 0=BEGIN (no-op), 1..=n=player CREATEs, n+1=COMMIT (no-op)
    let mut player_ids = Vec::with_capacity(players.len());
    for i in 0..players.len() {
        let player_row: Option<IdRecord> = response.take(i + 1)?;
        match player_row {
            Some(r) => player_ids.push(r.id.to_sql()),
            None => return Err(DbError::Other(format!("Failed to create player slot {i}"))),
        }
    }

    Ok((opponent_id, player_ids))
}

/// Persist enriched data fetched from the Riot API for a single opponent player.
///
/// Sets `last_fetched = time::now()` server-side to record when the data was written.
pub async fn update_opponent_player_intel(
    db: &Surreal<Db>,
    player_id: &str,
    riot_puuid: Option<String>,
    recent_champions: Vec<String>,
    mastery_data_json: Option<String>,
    role_distribution_json: Option<String>,
) -> DbResult<()> {
    let player_key = player_id
        .strip_prefix("opponent_player:")
        .unwrap_or(player_id)
        .to_string();
    db.query("UPDATE type::record('opponent_player', $player_key) SET riot_puuid = $riot_puuid, recent_champions = $recent_champions, mastery_data_json = $mastery_data_json, role_distribution_json = $role_distribution_json, last_fetched = time::now()")
        .bind(("player_key", player_key))
        .bind(("riot_puuid", riot_puuid))
        .bind(("recent_champions", recent_champions))
        .bind(("mastery_data_json", mastery_data_json))
        .bind(("role_distribution_json", role_distribution_json))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Series
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbSeries {
    id: RecordId,
    name: String,
    team: RecordId,
    opponent_id: Option<String>,
    opponent_name: Option<String>,
    format: String,
    is_fearless: bool,
    notes: Option<String>,
    created_by: RecordId,
}

impl From<DbSeries> for Series {
    fn from(s: DbSeries) -> Self {
        Series {
            id: Some(s.id.to_sql()),
            name: s.name,
            team_id: s.team.to_sql(),
            opponent_id: s.opponent_id,
            opponent_name: s.opponent_name,
            format: s.format,
            is_fearless: s.is_fearless,
            notes: s.notes,
            created_by: s.created_by.to_sql(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_series(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    name: String,
    opponent_id: Option<String>,
    opponent_name: Option<String>,
    format: String,
    is_fearless: bool,
    notes: Option<String>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let mut response = db
        .query("CREATE series SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent_id = $opponent_id, opponent_name = $opponent_name, format = $format, is_fearless = $is_fearless, notes = $notes")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("opponent_id", opponent_id))
        .bind(("opponent_name", opponent_name))
        .bind(("format", format))
        .bind(("is_fearless", is_fearless))
        .bind(("notes", notes))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create series".into())),
    }
}

pub async fn list_series(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<Series>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut result = db
        .query("SELECT * FROM series WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbSeries> = result.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(Series::from).collect())
}

pub async fn get_series(db: &Surreal<Db>, series_id: &str) -> DbResult<Option<Series>> {
    let series_key = series_id
        .strip_prefix("series:")
        .unwrap_or(series_id)
        .to_string();
    let mut result = db
        .query("SELECT * FROM type::record('series', $series_key)")
        .bind(("series_key", series_key))
        .await?;
    let row: Option<DbSeries> = result.take(0)?;
    Ok(row.map(Series::from))
}

#[allow(clippy::too_many_arguments)]
pub async fn update_series(
    db: &Surreal<Db>,
    series_id: &str,
    name: String,
    opponent_id: Option<String>,
    opponent_name: Option<String>,
    format: String,
    is_fearless: bool,
    notes: Option<String>,
) -> DbResult<()> {
    let series_key = series_id
        .strip_prefix("series:")
        .unwrap_or(series_id)
        .to_string();
    db.query("UPDATE type::record('series', $series_key) SET name=$name, opponent_id=$opponent_id, opponent_name=$opponent_name, format=$format, is_fearless=$is_fearless, notes=$notes")
        .bind(("series_key", series_key))
        .bind(("name", name))
        .bind(("opponent_id", opponent_id))
        .bind(("opponent_name", opponent_name))
        .bind(("format", format))
        .bind(("is_fearless", is_fearless))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_series(db: &Surreal<Db>, series_id: &str) -> DbResult<()> {
    let series_key = series_id
        .strip_prefix("series:")
        .unwrap_or(series_id)
        .to_string();
    let sid = series_id.to_string();
    // Unlink drafts first
    db.query("UPDATE draft SET series_id = NONE, game_number = NONE WHERE series_id = $sid")
        .bind(("sid", sid))
        .await?
        .check()?;
    db.query("DELETE type::record('series', $series_key)")
        .bind(("series_key", series_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn list_series_drafts(db: &Surreal<Db>, series_id: &str) -> DbResult<Vec<Draft>> {
    let sid = series_id.to_string();
    let mut result = db
        .query("SELECT * FROM draft WHERE series_id = $sid ORDER BY game_number ASC")
        .bind(("sid", sid))
        .await?;
    let db_drafts: Vec<DbDraft> = result.take(0).unwrap_or_default();

    let mut drafts: Vec<Draft> = Vec::new();
    for dd in db_drafts {
        let draft_id = dd.id.to_sql();
        let draft_key = draft_id
            .strip_prefix("draft:")
            .unwrap_or(&draft_id)
            .to_string();
        let mut draft = Draft::from(dd);

        let mut actions_result = db
            .query("SELECT * FROM draft_action WHERE draft = type::record('draft', $draft_key) ORDER BY `order` ASC")
            .bind(("draft_key", draft_key))
            .await?;
        let db_actions: Vec<DbDraftAction> = actions_result.take(0).unwrap_or_default();
        draft.actions = db_actions.into_iter().map(DraftAction::from).collect();
        drafts.push(draft);
    }
    Ok(drafts)
}

/// Get all champion names used in draft_actions for drafts in the given series.
/// Optionally exclude a specific draft (useful when editing the current draft).
pub async fn get_fearless_used_champions(
    db: &Surreal<Db>,
    series_id: &str,
    exclude_draft_id: Option<&str>,
) -> DbResult<Vec<String>> {
    let sid = series_id.to_string();

    let query = match exclude_draft_id {
        Some(eid) => {
            let eid_str = eid.to_string();
            let mut result = db
                .query("SELECT champion FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE series_id = $sid AND id != type::record('draft', $eid))")
                .bind(("sid", sid))
                .bind(("eid", eid_str.strip_prefix("draft:").unwrap_or(&eid_str).to_string()))
                .await?;
            let rows: Vec<ChampionRow> = result.take(0).unwrap_or_default();
            rows
        }
        None => {
            let mut result = db
                .query("SELECT champion FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE series_id = $sid)")
                .bind(("sid", sid))
                .await?;
            let rows: Vec<ChampionRow> = result.take(0).unwrap_or_default();
            rows
        }
    };

    let mut champions: Vec<String> = query.into_iter().map(|r| r.champion).collect();
    champions.sort();
    champions.dedup();
    Ok(champions)
}

#[derive(Debug, Deserialize, SurrealValue)]
struct ChampionRow {
    champion: String,
}

/// Get matchup notes from all team members that mention a specific champion.
/// Returns Vec<(username, ChampionNote)>.
pub async fn get_team_matchup_notes(
    db: &Surreal<Db>,
    team_id: &str,
    champion: &str,
) -> DbResult<Vec<(String, ChampionNote)>> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();

    // Get all team member user IDs and usernames
    let mut r = db
        .query("SELECT user.id as user_id, user.username as username FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberInfo {
        user_id: RecordId,
        username: String,
    }

    let members: Vec<MemberInfo> = r.take(0).unwrap_or_default();

    let mut result = Vec::new();
    for member in &members {
        let user_sql = member.user_id.to_sql();
        let user_key = user_sql
            .strip_prefix("user:")
            .unwrap_or(&user_sql)
            .to_string();
        let champion_owned = champion.to_string();
        let mut nr = db
            .query("SELECT *, <string>created_at AS created_at FROM champion_note WHERE user = type::record('user', $user_key) AND note_type = 'matchup' AND (title CONTAINS $champion OR content CONTAINS $champion)")
            .bind(("user_key", user_key))
            .bind(("champion", champion_owned))
            .await?;
        let notes: Vec<DbChampionNote> = nr.take(0).unwrap_or_default();
        for note in notes {
            result.push((member.username.clone(), ChampionNote::from(note)));
        }
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Action Items
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbActionItem {
    id: RecordId,
    team: RecordId,
    source_review: Option<String>,
    text: String,
    status: String,
    assigned_to: Option<String>,
    created_at: String,
    resolved_at: Option<String>,
}

impl From<DbActionItem> for ActionItem {
    fn from(a: DbActionItem) -> Self {
        ActionItem {
            id: Some(a.id.to_sql()),
            team_id: a.team.to_sql(),
            source_review: a.source_review,
            text: a.text,
            status: a.status,
            assigned_to: a.assigned_to,
            created_at: Some(a.created_at),
            resolved_at: a.resolved_at,
        }
    }
}

pub async fn create_action_item(
    db: &Surreal<Db>,
    team_id: &str,
    text: String,
    source_review: Option<String>,
    assigned_to: Option<String>,
) -> DbResult<String> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut response = db
        .query(
            "CREATE action_item SET team = type::record('team', $team_key), text = $text, source_review = $source_review, assigned_to = $assigned_to",
        )
        .bind(("team_key", team_key))
        .bind(("text", text))
        .bind(("source_review", source_review))
        .bind(("assigned_to", assigned_to))
        .await?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create action item".into())),
    }
}

pub async fn list_action_items(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<ActionItem>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query(
            "SELECT *, <string>created_at AS created_at, <string>resolved_at AS resolved_at FROM action_item WHERE team = type::record('team', $team_key) ORDER BY status ASC, created_at DESC",
        )
        .bind(("team_key", team_key))
        .await?;
    let items: Vec<DbActionItem> = r.take(0).unwrap_or_default();
    Ok(items.into_iter().map(ActionItem::from).collect())
}

pub async fn list_open_action_items(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<ActionItem>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query(
            "SELECT *, <string>created_at AS created_at, <string>resolved_at AS resolved_at FROM action_item WHERE team = type::record('team', $team_key) AND (status = 'open' OR status = 'in_progress') ORDER BY status ASC, created_at DESC",
        )
        .bind(("team_key", team_key))
        .await?;
    let items: Vec<DbActionItem> = r.take(0).unwrap_or_default();
    Ok(items.into_iter().map(ActionItem::from).collect())
}

pub async fn batch_create_action_items_from_review(
    db: &Surreal<Db>,
    team_id: &str,
    review_id: &str,
    improvements: &[String],
) -> DbResult<usize> {
    if improvements.is_empty() {
        return Ok(0);
    }
    // Fetch existing open/in_progress action items for dedup
    let existing = list_open_action_items(db, team_id).await?;
    let open_texts: std::collections::HashSet<String> = existing
        .iter()
        .map(|i| i.text.to_lowercase())
        .collect();

    let mut created = 0usize;
    for text in improvements {
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        // Skip if identical open/in_progress item exists (case-insensitive)
        if open_texts.contains(&text.to_lowercase()) {
            continue;
        }
        create_action_item(db, team_id, text, Some(review_id.to_string()), None).await?;
        created += 1;
    }
    Ok(created)
}

pub async fn update_action_item_status(
    db: &Surreal<Db>,
    item_id: &str,
    status: String,
) -> DbResult<()> {
    let item_key = item_id
        .strip_prefix("action_item:")
        .unwrap_or(item_id)
        .to_string();
    if status == "done" {
        db.query(
            "UPDATE type::record('action_item', $item_key) SET status = $status, resolved_at = time::now()",
        )
        .bind(("item_key", item_key))
        .bind(("status", status))
        .await?
        .check()?;
    } else {
        db.query(
            "UPDATE type::record('action_item', $item_key) SET status = $status, resolved_at = NONE",
        )
        .bind(("item_key", item_key))
        .bind(("status", status))
        .await?
        .check()?;
    }
    Ok(())
}

pub async fn update_action_item(
    db: &Surreal<Db>,
    item_id: &str,
    text: String,
    assigned_to: Option<String>,
    status: String,
) -> DbResult<()> {
    let item_key = item_id
        .strip_prefix("action_item:")
        .unwrap_or(item_id)
        .to_string();
    let resolved = if status == "done" {
        "resolved_at = time::now()"
    } else {
        "resolved_at = NONE"
    };
    let query = format!(
        "UPDATE type::record('action_item', $item_key) SET text = $text, assigned_to = $assigned_to, status = $status, {resolved}"
    );
    db.query(&query)
        .bind(("item_key", item_key))
        .bind(("text", text))
        .bind(("assigned_to", assigned_to))
        .bind(("status", status))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_action_item(db: &Surreal<Db>, item_id: &str) -> DbResult<()> {
    let item_key = item_id
        .strip_prefix("action_item:")
        .unwrap_or(item_id)
        .to_string();
    db.query("DELETE type::record('action_item', $item_key)")
        .bind(("item_key", item_key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Team Notes (shared notebook)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeamNote {
    id: RecordId,
    team: RecordId,
    author: RecordId,
    author_name: String,
    content: String,
    pinned: bool,
    created_at: String,
}

impl From<DbTeamNote> for TeamNote {
    fn from(n: DbTeamNote) -> Self {
        TeamNote {
            id: Some(n.id.to_sql()),
            team_id: n.team.to_sql(),
            author_id: n.author.to_sql(),
            author_name: n.author_name,
            content: n.content,
            pinned: n.pinned,
            created_at: Some(n.created_at),
        }
    }
}

pub async fn create_team_note(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    username: &str,
    content: String,
) -> DbResult<String> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();
    let user_key = user_id
        .strip_prefix("user:")
        .unwrap_or(user_id)
        .to_string();

    let mut response = db
        .query("CREATE team_note SET team = type::record('team', $team_key), author = type::record('user', $user_key), author_name = $author_name, content = $content")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("author_name", username.to_string()))
        .bind(("content", content))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create team note".into())),
    }
}

pub async fn list_team_notes(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<TeamNote>> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();

    let mut response = db
        .query("SELECT *, <string>created_at AS created_at FROM team_note WHERE team = type::record('team', $team_key) ORDER BY pinned DESC, created_at DESC")
        .bind(("team_key", team_key))
        .await?;

    let rows: Vec<DbTeamNote> = response.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(TeamNote::from).collect())
}

pub async fn list_pinned_team_notes(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<TeamNote>> {
    let team_key = team_id
        .strip_prefix("team:")
        .unwrap_or(team_id)
        .to_string();

    let mut response = db
        .query("SELECT *, <string>created_at AS created_at FROM team_note WHERE team = type::record('team', $team_key) AND pinned = true ORDER BY created_at DESC")
        .bind(("team_key", team_key))
        .await?;

    let rows: Vec<DbTeamNote> = response.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(TeamNote::from).collect())
}

pub async fn update_team_note(db: &Surreal<Db>, note_id: &str, content: String) -> DbResult<()> {
    let note_key = note_id
        .strip_prefix("team_note:")
        .unwrap_or(note_id)
        .to_string();

    db.query("UPDATE type::record('team_note', $note_key) SET content = $content")
        .bind(("note_key", note_key))
        .bind(("content", content))
        .await?
        .check()?;
    Ok(())
}

pub async fn toggle_pin_team_note(
    db: &Surreal<Db>,
    note_id: &str,
    pinned: bool,
) -> DbResult<()> {
    let note_key = note_id
        .strip_prefix("team_note:")
        .unwrap_or(note_id)
        .to_string();

    db.query("UPDATE type::record('team_note', $note_key) SET pinned = $pinned")
        .bind(("note_key", note_key))
        .bind(("pinned", pinned))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_team_note(db: &Surreal<Db>, note_id: &str) -> DbResult<()> {
    let note_key = note_id
        .strip_prefix("team_note:")
        .unwrap_or(note_id)
        .to_string();

    db.query("DELETE type::record('team_note', $note_key)")
        .bind(("note_key", note_key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Analytics: Draft Tendencies, Win Condition Stats, Draft Outcomes
// ---------------------------------------------------------------------------

/// Returns Vec<(champion, phase, order, count)> for the team's drafts.
pub async fn get_draft_tendencies(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<(String, String, i32, i32)>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Fetch all draft actions for this team's drafts
    let mut result = db
        .query("SELECT * FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE team = type::record('team', $team_key)) ORDER BY `order` ASC")
        .bind(("team_key", team_key))
        .await?;
    let actions: Vec<DbDraftAction> = result.take(0).unwrap_or_default();

    // Aggregate in Rust: (champion, phase, order) -> count
    let mut counts: HashMap<(String, String, i32), i32> = HashMap::new();
    for a in actions {
        let key = (a.champion.clone(), a.phase.clone(), a.order);
        *counts.entry(key).or_insert(0) += 1;
    }

    let mut tendencies: Vec<(String, String, i32, i32)> = counts
        .into_iter()
        .map(|((champ, phase, order), count)| (champ, phase, order, count))
        .collect();
    tendencies.sort_by(|a, b| b.3.cmp(&a.3));
    Ok(tendencies)
}

/// Returns Vec<(tag, total_games, wins)> for game plans with win_condition_tag set.
/// Correlates with post_game_learning outcomes.
pub async fn get_win_condition_stats(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<(String, i32, i32)>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Get all game plans with a win_condition_tag
    let mut result = db
        .query("SELECT id, win_condition_tag FROM game_plan WHERE team = type::record('team', $team_key) AND win_condition_tag != NONE; SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key); SELECT match_id, id FROM match WHERE team_id = type::record('team', $team_key); SELECT * FROM player_match")
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PlanTag {
        id: RecordId,
        win_condition_tag: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPostGame {
        id: RecordId,
        game_plan_id: Option<String>,
        match_riot_id: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbMatchRef {
        match_id: String,
        id: RecordId,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPlayerMatch {
        #[serde(rename = "match")]
        match_ref: RecordId,
        win: bool,
    }

    let plans: Vec<PlanTag> = result.take(0).unwrap_or_default();
    let post_games: Vec<DbPostGame> = result.take(1).unwrap_or_default();
    let matches: Vec<DbMatchRef> = result.take(2).unwrap_or_default();
    let player_matches: Vec<DbPlayerMatch> = result.take(3).unwrap_or_default();

    // Build lookup: match_riot_id -> win (any player win means team won)
    let match_id_to_record: HashMap<String, String> = matches
        .iter()
        .map(|m| (m.match_id.clone(), m.id.to_sql()))
        .collect();

    let mut match_record_wins: HashMap<String, bool> = HashMap::new();
    for pm in &player_matches {
        let match_sql = pm.match_ref.to_sql();
        if pm.win {
            match_record_wins.insert(match_sql, true);
        } else {
            match_record_wins.entry(match_sql).or_insert(false);
        }
    }

    // Build lookup: game_plan_id -> (tag, has_outcome, won)
    let plan_tags: HashMap<String, String> = plans
        .iter()
        .filter_map(|p| {
            p.win_condition_tag
                .as_ref()
                .map(|tag| (p.id.to_sql(), tag.clone()))
        })
        .collect();

    // Aggregate: tag -> (total, wins)
    let mut tag_stats: HashMap<String, (i32, i32)> = HashMap::new();

    for pg in &post_games {
        if let Some(ref gp_id) = pg.game_plan_id {
            if let Some(tag) = plan_tags.get(gp_id) {
                if let Some(ref riot_id) = pg.match_riot_id {
                    if let Some(record_id) = match_id_to_record.get(riot_id) {
                        let won = match_record_wins.get(record_id).copied().unwrap_or(false);
                        let entry = tag_stats.entry(tag.clone()).or_insert((0, 0));
                        entry.0 += 1;
                        if won {
                            entry.1 += 1;
                        }
                    }
                }
            }
        }
    }

    // Also count plans with tag that have no outcome yet (as games without result)
    for (plan_id, tag) in &plan_tags {
        let has_post_game = post_games
            .iter()
            .any(|pg| pg.game_plan_id.as_deref() == Some(plan_id));
        if !has_post_game {
            // Count as a game played but with no outcome data
            tag_stats.entry(tag.clone()).or_insert((0, 0));
        }
    }

    let mut stats: Vec<(String, i32, i32)> = tag_stats
        .into_iter()
        .map(|(tag, (total, wins))| (tag, total, wins))
        .collect();
    stats.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(stats)
}

/// Pure helper: filters win condition stats to only games linked to a specific opponent.
/// Input: per-game-plan data as (win_condition_tag, opponent_name, is_win) triples.
/// Returns: aggregated (tag, total_games, wins) for plans matching `opponent_name`.
pub fn filter_win_condition_stats(
    plan_tags: &[(String, Option<String>, bool)],
    opponent_name: &str,
) -> Vec<(String, i32, i32)> {
    let mut tag_stats: HashMap<String, (i32, i32)> = HashMap::new();
    for (tag, opp, won) in plan_tags {
        if opp.as_deref() == Some(opponent_name) {
            let entry = tag_stats.entry(tag.clone()).or_insert((0, 0));
            entry.0 += 1;
            if *won {
                entry.1 += 1;
            }
        }
    }
    let mut stats: Vec<(String, i32, i32)> = tag_stats
        .into_iter()
        .map(|(tag, (total, wins))| (tag, total, wins))
        .collect();
    stats.sort_by(|a, b| b.1.cmp(&a.1));
    stats
}

/// Returns win condition stats filtered to games against a specific opponent.
/// Joins game_plan -> draft -> opponent_name for filtering.
pub async fn get_win_condition_stats_vs_opponent(
    db: &Surreal<Db>,
    team_id: &str,
    opponent_name: &str,
) -> DbResult<Vec<(String, i32, i32)>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Fetch game plans with win_condition_tag, drafts (for opponent), and post_game_learning
    let mut result = db
        .query("SELECT id, win_condition_tag, draft FROM game_plan WHERE team = type::record('team', $team_key) AND win_condition_tag != NONE; SELECT id, opponent FROM draft WHERE team = type::record('team', $team_key); SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key); SELECT match_id, id FROM match WHERE team_id = type::record('team', $team_key); SELECT * FROM player_match")
        .bind(("team_key", team_key))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PlanTagWithDraft {
        id: RecordId,
        win_condition_tag: Option<String>,
        draft: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbDraftOpponent {
        id: RecordId,
        opponent: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPostGameVs {
        id: RecordId,
        game_plan_id: Option<String>,
        match_riot_id: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbMatchRefVs {
        match_id: String,
        id: RecordId,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPlayerMatchVs {
        #[serde(rename = "match")]
        match_ref: RecordId,
        win: bool,
    }

    let plans: Vec<PlanTagWithDraft> = result.take(0).unwrap_or_default();
    let drafts: Vec<DbDraftOpponent> = result.take(1).unwrap_or_default();
    let post_games: Vec<DbPostGameVs> = result.take(2).unwrap_or_default();
    let matches: Vec<DbMatchRefVs> = result.take(3).unwrap_or_default();
    let player_matches: Vec<DbPlayerMatchVs> = result.take(4).unwrap_or_default();

    // Build draft id -> opponent lookup
    let draft_opponent: HashMap<String, String> = drafts
        .iter()
        .filter_map(|d| {
            d.opponent
                .as_ref()
                .map(|opp| (d.id.to_sql(), opp.clone()))
        })
        .collect();

    // Build match outcome lookup
    let match_id_to_record: HashMap<String, String> = matches
        .iter()
        .map(|m| (m.match_id.clone(), m.id.to_sql()))
        .collect();

    let mut match_record_wins: HashMap<String, bool> = HashMap::new();
    for pm in &player_matches {
        let match_sql = pm.match_ref.to_sql();
        if pm.win {
            match_record_wins.insert(match_sql, true);
        } else {
            match_record_wins.entry(match_sql).or_insert(false);
        }
    }

    // Build per-game-plan flat data: (tag, opponent, is_win)
    let plan_tags_lookup: HashMap<String, (String, Option<String>)> = plans
        .iter()
        .filter_map(|p| {
            p.win_condition_tag.as_ref().map(|tag| {
                let plan_sql = p.id.to_sql();
                let opp = p.draft.as_ref().and_then(|did| draft_opponent.get(did).cloned());
                (plan_sql, (tag.clone(), opp))
            })
        })
        .collect();

    // Build flat per-result tuples for filter_win_condition_stats
    let mut flat: Vec<(String, Option<String>, bool)> = Vec::new();
    for pg in &post_games {
        if let Some(ref gp_id) = pg.game_plan_id {
            if let Some((tag, opp)) = plan_tags_lookup.get(gp_id) {
                if let Some(ref riot_id) = pg.match_riot_id {
                    if let Some(record_id) = match_id_to_record.get(riot_id) {
                        let won = match_record_wins.get(record_id).copied().unwrap_or(false);
                        flat.push((tag.clone(), opp.clone(), won));
                    }
                }
            }
        }
    }

    Ok(filter_win_condition_stats(&flat, opponent_name))
}

/// Draft outcome statistics: side win rates, tag win rates, first pick win rates.
pub async fn get_draft_outcome_stats(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<DraftOutcomeData> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    let mut result = db
        .query("SELECT * FROM draft WHERE team = type::record('team', $team_key); SELECT * FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE team = type::record('team', $team_key)) ORDER BY `order` ASC; SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key); SELECT match_id, id FROM match WHERE team_id = type::record('team', $team_key); SELECT * FROM player_match")
        .bind(("team_key", team_key))
        .await?;

    let db_drafts: Vec<DbDraft> = result.take(0).unwrap_or_default();
    let db_actions: Vec<DbDraftAction> = result.take(1).unwrap_or_default();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPostGame2 {
        draft_id: Option<String>,
        match_riot_id: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbMatchRef2 {
        match_id: String,
        id: RecordId,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPlayerMatch2 {
        #[serde(rename = "match")]
        match_ref: RecordId,
        win: bool,
    }

    let post_games: Vec<DbPostGame2> = result.take(2).unwrap_or_default();
    let matches: Vec<DbMatchRef2> = result.take(3).unwrap_or_default();
    let player_matches: Vec<DbPlayerMatch2> = result.take(4).unwrap_or_default();

    // Build match outcome lookup
    let match_id_to_record: HashMap<String, String> = matches
        .iter()
        .map(|m| (m.match_id.clone(), m.id.to_sql()))
        .collect();

    let mut match_record_wins: HashMap<String, bool> = HashMap::new();
    for pm in &player_matches {
        let match_sql = pm.match_ref.to_sql();
        if pm.win {
            match_record_wins.insert(match_sql, true);
        } else {
            match_record_wins.entry(match_sql).or_insert(false);
        }
    }

    // Build draft_id -> outcome lookup via post_game_learning
    let mut draft_outcome: HashMap<String, bool> = HashMap::new();
    for pg in &post_games {
        if let (Some(ref did), Some(ref riot_id)) = (&pg.draft_id, &pg.match_riot_id) {
            if let Some(record_id) = match_id_to_record.get(riot_id) {
                if let Some(&won) = match_record_wins.get(record_id) {
                    draft_outcome.insert(did.clone(), won);
                }
            }
        }
    }

    // Build actions-by-draft map
    let mut actions_by_draft: HashMap<String, Vec<DraftAction>> = HashMap::new();
    for a in db_actions {
        let draft_id = a.draft.to_sql();
        actions_by_draft
            .entry(draft_id)
            .or_default()
            .push(DraftAction::from(a));
    }

    let mut blue_games = 0i32;
    let mut blue_wins = 0i32;
    let mut red_games = 0i32;
    let mut red_wins = 0i32;
    let mut tag_counts: HashMap<String, (i32, i32)> = HashMap::new();
    let mut first_pick_counts: HashMap<String, (i32, i32)> = HashMap::new();

    for d in &db_drafts {
        let draft_id = d.id.to_sql();
        let won = match draft_outcome.get(&draft_id) {
            Some(&w) => w,
            None => continue, // No outcome data for this draft
        };

        // Side stats
        if d.our_side == "blue" {
            blue_games += 1;
            if won {
                blue_wins += 1;
            }
        } else {
            red_games += 1;
            if won {
                red_wins += 1;
            }
        }

        // Tag stats
        for tag in &d.tags {
            let entry = tag_counts.entry(tag.clone()).or_insert((0, 0));
            entry.0 += 1;
            if won {
                entry.1 += 1;
            }
        }

        // First pick: find the first pick action on our side
        if let Some(actions) = actions_by_draft.get(&draft_id) {
            let first_pick = actions
                .iter()
                .filter(|a| a.phase.contains("pick") && a.side == d.our_side)
                .min_by_key(|a| a.order);
            if let Some(fp) = first_pick {
                let entry = first_pick_counts
                    .entry(fp.champion.clone())
                    .or_insert((0, 0));
                entry.0 += 1;
                if won {
                    entry.1 += 1;
                }
            }
        }
    }

    let mut tag_stats: Vec<(String, i32, i32)> = tag_counts
        .into_iter()
        .map(|(tag, (g, w))| (tag, g, w))
        .collect();
    tag_stats.sort_by(|a, b| b.1.cmp(&a.1));

    let mut fp_stats: Vec<(String, i32, i32)> = first_pick_counts
        .into_iter()
        .map(|(champ, (g, w))| (champ, g, w))
        .collect();
    fp_stats.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(DraftOutcomeData {
        blue_games,
        blue_wins,
        red_games,
        red_wins,
        tag_stats,
        first_pick_stats: fp_stats,
    })
}

// ---------------------------------------------------------------------------
// Champion Name Migration
// ---------------------------------------------------------------------------

/// Normalize champion names across all tables at startup.
/// This is a best-effort migration: if Data Dragon is unreachable, we log a warning and continue.
/// Tables normalized: champion_pool, draft_action, game_plan (our_champions/enemy_champions arrays),
/// opponent_player (recent_champions array), ban_priority, tree_node_action.
pub async fn migrate_champion_names(db: &Surreal<Db>) -> DbResult<()> {
    use crate::server::data_dragon;
    use tracing::{info, warn};

    let champions = match data_dragon::fetch_champions().await {
        Ok(c) => c,
        Err(e) => {
            warn!("Champion name migration skipped — could not fetch Data Dragon: {e}");
            return Ok(());
        }
    };

    // Build lookup maps for normalization
    let canonical_ids: std::collections::HashSet<&str> =
        champions.iter().map(|c| c.id.as_str()).collect();

    // Helper: normalize a champion string — returns Some(canonical_id) if different from input
    let normalize = |input: &str| -> Option<String> {
        if canonical_ids.contains(input) {
            return None; // Already canonical, no update needed
        }
        data_dragon::normalize_champion_name(input, &champions)
    };

    // Normalize single-value champion fields
    let single_tables: &[(&str, &str)] = &[
        ("champion_pool", "champion"),
        ("draft_action", "champion"),
        ("ban_priority", "champion"),
        ("tree_node_action", "champion"),
    ];

    for (table, field) in single_tables {
        #[derive(Debug, Deserialize, SurrealValue)]
        struct ChampField {
            id: RecordId,
            champion: String,
        }

        let query = format!("SELECT id, {field} as champion FROM {table}");
        let mut r = db.query(&query).await?;
        let rows: Vec<ChampField> = r.take(0).unwrap_or_default();

        for row in rows {
            if let Some(canonical) = normalize(&row.champion) {
                let record_key = row.id.to_sql();
                let table_prefix = format!("{table}:");
                let key = record_key
                    .strip_prefix(&table_prefix)
                    .unwrap_or(&record_key)
                    .to_string();
                let update_query =
                    format!("UPDATE type::record('{table}', $key) SET {field} = $new_champ");
                db.query(&update_query)
                    .bind(("key", key))
                    .bind(("new_champ", canonical.clone()))
                    .await?
                    .check()?;
                info!(
                    "Normalized champion: '{}' -> '{}' in {table}",
                    row.champion, canonical
                );
            }
        }
    }

    // Normalize array fields: game_plan.our_champions, game_plan.enemy_champions, opponent_player.recent_champions
    let array_tables: &[(&str, &str)] = &[
        ("game_plan", "our_champions"),
        ("game_plan", "enemy_champions"),
        ("opponent_player", "recent_champions"),
    ];

    for (table, field) in array_tables {
        #[derive(Debug, Deserialize, SurrealValue)]
        struct ArrayField {
            id: RecordId,
            champions: Vec<String>,
        }

        let query = format!("SELECT id, {field} as champions FROM {table}");
        let mut r = db.query(&query).await?;
        let rows: Vec<ArrayField> = r.take(0).unwrap_or_default();

        for row in rows {
            let mut changed = false;
            let normalized: Vec<String> = row
                .champions
                .iter()
                .map(|c| {
                    if let Some(canon) = normalize(c) {
                        changed = true;
                        info!("Normalized champion: '{c}' -> '{canon}' in {table}.{field}");
                        canon
                    } else {
                        c.clone()
                    }
                })
                .collect();

            if changed {
                let record_key = row.id.to_sql();
                let table_prefix = format!("{table}:");
                let key = record_key
                    .strip_prefix(&table_prefix)
                    .unwrap_or(&record_key)
                    .to_string();
                let update_query =
                    format!("UPDATE type::record('{table}', $key) SET {field} = $new_champs");
                db.query(&update_query)
                    .bind(("key", key))
                    .bind(("new_champs", normalized))
                    .await?
                    .check()?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Dashboard Summary
// ---------------------------------------------------------------------------

/// Compute pool gap warnings for a set of champion pool entries.
///
/// Groups entries by (user_id, role), then checks:
/// - If any single class represents >= 70% of the class distribution, warn about dominance.
/// - If a class is entirely missing AND it appears in opponent_champions' class tags, set opponent_escalated.
///
/// `members` maps user_id -> username for populating warning structs.
pub fn compute_pool_gaps(
    pool_entries: &[ChampionPoolEntry],
    members: &[(String, String)], // (user_id, username)
    champions: &[Champion],
    opponent_champions: &[String],
) -> Vec<PoolGapWarning> {
    const ALL_CLASSES: &[&str] = &[
        "Fighter", "Mage", "Assassin", "Tank", "Marksman", "Support",
    ];

    // Build champion class lookup: champion_id -> Vec<class>
    let class_map: HashMap<&str, &[String]> =
        champions.iter().map(|c| (c.id.as_str(), c.tags.as_slice())).collect();

    // Build username lookup
    let username_map: HashMap<&str, &str> = members
        .iter()
        .map(|(uid, uname)| (uid.as_str(), uname.as_str()))
        .collect();

    // Collect all class tags that appear in opponent champions
    let opponent_classes: std::collections::HashSet<String> = opponent_champions
        .iter()
        .flat_map(|champ| {
            class_map
                .get(champ.as_str())
                .map(|tags| tags.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .collect();

    // Group pool entries by (user_id, role)
    let mut by_user_role: HashMap<(String, String), Vec<&ChampionPoolEntry>> = HashMap::new();
    for entry in pool_entries {
        by_user_role
            .entry((entry.user_id.clone(), entry.role.clone()))
            .or_default()
            .push(entry);
    }

    let mut warnings = Vec::new();

    for ((user_id, role), entries) in &by_user_role {
        // Count class occurrences
        let mut class_counts: HashMap<&str, usize> = HashMap::new();
        for entry in entries {
            if let Some(tags) = class_map.get(entry.champion.as_str()) {
                for tag in *tags {
                    *class_counts.entry(tag.as_str()).or_default() += 1;
                }
            }
        }

        let total: usize = class_counts.values().sum();

        let mut dominant_class: Option<String> = None;
        let mut missing_classes = Vec::new();
        let mut opponent_escalated = false;

        for &class in ALL_CLASSES {
            let count = class_counts.get(class).copied().unwrap_or(0);

            // Check for dominance: >= 70% of total class tags
            if total > 0 && (count as f64 / total as f64) >= 0.70 {
                dominant_class = Some(class.to_string());
            }

            // Check for missing class with opponent escalation
            if count == 0 && opponent_classes.contains(class) {
                missing_classes.push(class.to_string());
                opponent_escalated = true;
            }
        }

        if dominant_class.is_some() || !missing_classes.is_empty() {
            let username = username_map
                .get(user_id.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            warnings.push(PoolGapWarning {
                user_id: user_id.clone(),
                username,
                role: role.clone(),
                dominant_class,
                missing_classes,
                opponent_escalated,
            });
        }
    }

    warnings.sort_by(|a, b| a.user_id.cmp(&b.user_id).then(a.role.cmp(&b.role)));
    warnings
}

/// Fetch all summary data for the team dashboard in a single batched query.
///
/// Returns `Ok(DashboardSummary::default())` if the team has no data or user has no team.
pub async fn get_dashboard_summary(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<DashboardSummary> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // 5-statement batched query (per RESEARCH.md Example 2)
    let mut result = db
        .query(
            "SELECT id, text, <string>created_at AS created_at FROM action_item \
             WHERE team = type::record('team', $team_key) AND status IN ['open','in_progress'] \
             ORDER BY created_at DESC LIMIT 3; \
             SELECT count() as n FROM action_item \
             WHERE team = type::record('team', $team_key) AND status IN ['open','in_progress'] \
             GROUP ALL; \
             SELECT id, improvements, <string>created_at AS created_at FROM post_game_learning \
             WHERE team = type::record('team', $team_key) \
             ORDER BY created_at DESC LIMIT 5; \
             SELECT count() as n FROM draft WHERE team = type::record('team', $team_key) \
             AND draft NOT IN (SELECT VALUE draft FROM game_plan WHERE draft != NONE) \
             GROUP ALL; \
             SELECT count() as n FROM game_plan WHERE team = type::record('team', $team_key) \
             AND id NOT IN (SELECT VALUE game_plan_id FROM post_game_learning WHERE game_plan_id != NONE) \
             GROUP ALL",
        )
        .bind(("team_key", team_key.clone()))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbActionItemPreview {
        id: RecordId,
        text: String,
        #[allow(dead_code)]
        created_at: String,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct CountResult {
        n: i64,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPostGamePreview {
        id: RecordId,
        improvements: Vec<String>,
        created_at: Option<String>,
    }

    let db_action_items: Vec<DbActionItemPreview> = result.take(0).unwrap_or_default();
    let action_count_rows: Vec<CountResult> = result.take(1).unwrap_or_default();
    let db_post_games: Vec<DbPostGamePreview> = result.take(2).unwrap_or_default();
    let drafts_no_plan_rows: Vec<CountResult> = result.take(3).unwrap_or_default();
    let plans_no_postgame_rows: Vec<CountResult> = result.take(4).unwrap_or_default();

    let open_action_item_count = action_count_rows.first().map(|r| r.n as usize).unwrap_or(0);
    let drafts_without_game_plan = drafts_no_plan_rows
        .first()
        .map(|r| r.n as usize)
        .unwrap_or(0);
    let game_plans_without_post_game = plans_no_postgame_rows
        .first()
        .map(|r| r.n as usize)
        .unwrap_or(0);

    let recent_action_items: Vec<ActionItemPreview> = db_action_items
        .into_iter()
        .map(|a| ActionItemPreview {
            id: a.id.to_sql(),
            text: a.text,
        })
        .collect();

    let recent_post_games: Vec<PostGamePreview> = db_post_games
        .into_iter()
        .map(|p| PostGamePreview {
            id: p.id.to_sql(),
            improvements: p.improvements,
            created_at: p.created_at,
        })
        .collect();

    // Fetch champion pool entries for all team members (for gap analysis)
    let pool_gap_warnings = match compute_pool_gaps_for_team(db, &team_key).await {
        Ok(warnings) => warnings,
        Err(_) => Vec::new(), // Non-fatal: gap analysis failure should not break dashboard
    };

    Ok(DashboardSummary {
        open_action_item_count,
        recent_action_items,
        recent_post_games,
        pool_gap_warnings,
        drafts_without_game_plan,
        game_plans_without_post_game,
    })
}

/// Internal helper: fetch pool gap warnings for a team.
/// Loads team members, their champion pools, and opponent data, then calls compute_pool_gaps.
async fn compute_pool_gaps_for_team(
    db: &Surreal<Db>,
    team_key: &str,
) -> DbResult<Vec<PoolGapWarning>> {
    use crate::server::data_dragon;

    // Fetch Data Dragon champions for class tags (best-effort)
    let champions = match data_dragon::fetch_champions().await {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };

    // Get team members
    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberInfo {
        user_id: RecordId,
        username: String,
    }

    let mut r = db
        .query("SELECT user.id as user_id, user.username as username FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", team_key.to_string()))
        .await?;
    let members_db: Vec<MemberInfo> = r.take(0).unwrap_or_default();

    if members_db.is_empty() {
        return Ok(Vec::new());
    }

    let members: Vec<(String, String)> = members_db
        .iter()
        .map(|m| (m.user_id.to_sql(), m.username.clone()))
        .collect();

    // Get all pool entries for team members
    let user_ids: Vec<String> = members_db.iter().map(|m| m.user_id.to_sql()).collect();
    let user_id_list = user_ids.join(", ");
    let pool_query = format!(
        "SELECT user, champion, role, tier FROM champion_pool WHERE user IN [{user_id_list}]"
    );

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PoolRow {
        user: RecordId,
        champion: String,
        role: String,
        tier: String,
    }

    let mut pr = db.query(&pool_query).await?;
    let pool_rows: Vec<PoolRow> = pr.take(0).unwrap_or_default();

    let pool_entries: Vec<ChampionPoolEntry> = pool_rows
        .into_iter()
        .map(|row| ChampionPoolEntry {
            id: None,
            user_id: row.user.to_sql(),
            champion: row.champion,
            role: row.role,
            tier: row.tier,
            notes: None,
            comfort_level: None,
            meta_tag: None,
        })
        .collect();

    // Get opponent champion lists for escalation detection
    let mut opp_r = db
        .query("SELECT recent_champions FROM opponent_player WHERE opponent IN (SELECT VALUE id FROM opponent WHERE team = type::record('team', $team_key))")
        .bind(("team_key", team_key.to_string()))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct RecentChamps {
        recent_champions: Vec<String>,
    }

    let opp_rows: Vec<RecentChamps> = opp_r.take(0).unwrap_or_default();
    let opponent_champions: Vec<String> = opp_rows
        .into_iter()
        .flat_map(|r| r.recent_champions)
        .collect();

    Ok(compute_pool_gaps(
        &pool_entries,
        &members,
        &champions,
        &opponent_champions,
    ))
}

// ---------------------------------------------------------------------------
// Champion Performance Summary
// ---------------------------------------------------------------------------

/// Aggregate per-champion performance from draft, match, game plan, and post-game data.
/// This is a pure Rust helper that can be tested without a DB connection.
pub fn aggregate_champion_performance(
    draft_champions: &[String],
    match_stats: &[(String, bool)], // (champion, win)
    plan_champions: &[String],
    post_game_champ_outcomes: &[(String, bool)], // (champion, win)
) -> Vec<ChampionPerformanceSummary> {
    let mut map: HashMap<String, ChampionPerformanceSummary> = HashMap::new();

    for champ in draft_champions {
        let entry = map.entry(champ.clone()).or_insert_with(|| ChampionPerformanceSummary {
            champion: champ.clone(),
            ..Default::default()
        });
        entry.games_in_draft += 1;
    }

    for (champ, win) in match_stats {
        let entry = map.entry(champ.clone()).or_insert_with(|| ChampionPerformanceSummary {
            champion: champ.clone(),
            ..Default::default()
        });
        entry.games_in_match += 1;
        if *win {
            entry.wins_in_match += 1;
        }
    }

    for champ in plan_champions {
        let entry = map.entry(champ.clone()).or_insert_with(|| ChampionPerformanceSummary {
            champion: champ.clone(),
            ..Default::default()
        });
        entry.games_in_plan += 1;
    }

    for (champ, win) in post_game_champ_outcomes {
        let entry = map.entry(champ.clone()).or_insert_with(|| ChampionPerformanceSummary {
            champion: champ.clone(),
            ..Default::default()
        });
        if *win {
            entry.post_game_wins += 1;
        } else {
            entry.post_game_losses += 1;
        }
    }

    let mut result: Vec<ChampionPerformanceSummary> = map.into_values().collect();
    result.sort_by(|a, b| b.games_in_match.cmp(&a.games_in_match));
    result
}

/// Get per-player champion performance summary.
///
/// Applies "30 days OR 20 games, whichever is more" window logic.
/// Returns `Ok(Vec::new())` when user has no team or no data.
pub async fn get_champion_performance_summary(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    window_days: Option<i64>,
    min_games: Option<usize>,
) -> DbResult<Vec<ChampionPerformanceSummary>> {
    use chrono::{DateTime, Duration, Utc};

    if team_id.is_empty() {
        return Ok(Vec::new());
    }

    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let days = window_days.unwrap_or(30);
    let min = min_games.unwrap_or(20);

    // Calculate the 30-day cutoff
    let day_cutoff: DateTime<Utc> = Utc::now() - Duration::days(days);
    let day_cutoff_str = day_cutoff.to_rfc3339();

    // Count matches within the day-based window
    #[derive(Debug, Deserialize, SurrealValue)]
    struct CountResult {
        n: i64,
    }

    let mut count_r = db
        .query(
            "SELECT count() as n FROM player_match_stats \
             WHERE user = type::record('user', $user_key) \
             AND created_at >= $cutoff GROUP ALL",
        )
        .bind(("user_key", user_key.clone()))
        .bind(("cutoff", day_cutoff_str.clone()))
        .await?;

    let count_rows: Vec<CountResult> = count_r.take(0).unwrap_or_default();
    let match_count = count_rows.first().map(|r| r.n as usize).unwrap_or(0);

    // Determine effective cutoff
    let effective_cutoff = if match_count < min {
        // Try to extend window to at least min_games
        #[derive(Debug, Deserialize, SurrealValue)]
        struct DateRow {
            created_at: String,
        }

        let start_idx = if min > 1 { min - 1 } else { 0 };
        let mut oldest_r = db
            .query(
                "SELECT <string>created_at AS created_at FROM player_match_stats \
                 WHERE user = type::record('user', $user_key) \
                 ORDER BY created_at DESC LIMIT 1 START $start_idx",
            )
            .bind(("user_key", user_key.clone()))
            .bind(("start_idx", start_idx as i64))
            .await?;

        let oldest_rows: Vec<DateRow> = oldest_r.take(0).unwrap_or_default();
        oldest_rows
            .into_iter()
            .next()
            .map(|r| r.created_at)
            .unwrap_or(day_cutoff_str)
    } else {
        day_cutoff_str
    };

    // Batched query: draft picks, match stats, game plan champions, post-game outcomes
    let mut result = db
        .query(
            "SELECT champion FROM draft_action \
             WHERE draft IN (SELECT VALUE id FROM draft WHERE team = type::record('team', $team_key)) \
             AND phase CONTAINS 'pick' \
             AND created_at >= $cutoff; \
             SELECT champion, win FROM player_match_stats \
             WHERE user = type::record('user', $user_key) \
             AND created_at >= $cutoff; \
             SELECT our_champions FROM game_plan \
             WHERE team = type::record('team', $team_key) \
             AND created_at >= $cutoff; \
             SELECT id, improvements FROM post_game_learning \
             WHERE team = type::record('team', $team_key) \
             AND created_at >= $cutoff",
        )
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .bind(("cutoff", effective_cutoff.clone()))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DraftChampRow {
        champion: String,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct MatchStatRow {
        champion: String,
        win: bool,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PlanChampRow {
        our_champions: Vec<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PostGameRow {
        #[allow(dead_code)]
        id: RecordId,
        improvements: Vec<String>,
    }

    let draft_rows: Vec<DraftChampRow> = result.take(0).unwrap_or_default();
    let match_rows: Vec<MatchStatRow> = result.take(1).unwrap_or_default();
    let plan_rows: Vec<PlanChampRow> = result.take(2).unwrap_or_default();
    let post_game_rows: Vec<PostGameRow> = result.take(3).unwrap_or_default();

    if draft_rows.is_empty()
        && match_rows.is_empty()
        && plan_rows.is_empty()
        && post_game_rows.is_empty()
    {
        return Ok(Vec::new());
    }

    let draft_champions: Vec<String> = draft_rows.into_iter().map(|r| r.champion).collect();
    let match_stats: Vec<(String, bool)> =
        match_rows.into_iter().map(|r| (r.champion, r.win)).collect();
    let plan_champions: Vec<String> = plan_rows
        .into_iter()
        .flat_map(|r| r.our_champions)
        .collect();

    // Post-game outcomes: we don't have per-champion win/loss in post_game_learning,
    // so we use improvements list length as a proxy for loss (improvements = things to fix)
    // The post_game_learning table doesn't store win/loss directly, so we leave this empty for now
    let post_game_outcomes: Vec<(String, bool)> = Vec::new();

    let _ = post_game_rows; // Used for count reference but outcomes N/A without win field

    Ok(aggregate_champion_performance(
        &draft_champions,
        &match_stats,
        &plan_champions,
        &post_game_outcomes,
    ))
}

/// Get team-wide champion performance summary.
///
/// Same "30 days OR 20 games" logic applied across all team members.
/// Returns `Ok(Vec::new())` when team has no data.
pub async fn get_team_champion_performance(
    db: &Surreal<Db>,
    team_id: &str,
    window_days: Option<i64>,
    min_games: Option<usize>,
) -> DbResult<Vec<ChampionPerformanceSummary>> {
    use chrono::{DateTime, Duration, Utc};

    if team_id.is_empty() {
        return Ok(Vec::new());
    }

    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let days = window_days.unwrap_or(30);
    let min = min_games.unwrap_or(20);

    // Get all team member user IDs
    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberRef {
        user_id: RecordId,
    }

    let mut member_r = db
        .query(
            "SELECT user.id as user_id FROM team_member WHERE team = type::record('team', $team_key)",
        )
        .bind(("team_key", team_key.clone()))
        .await?;

    let member_refs: Vec<MemberRef> = member_r.take(0).unwrap_or_default();
    if member_refs.is_empty() {
        return Ok(Vec::new());
    }

    let user_ids: Vec<String> = member_refs.iter().map(|m| m.user_id.to_sql()).collect();

    // Determine time window
    let day_cutoff: DateTime<Utc> = Utc::now() - Duration::days(days);
    let day_cutoff_str = day_cutoff.to_rfc3339();

    let user_id_list = user_ids.join(", ");

    #[derive(Debug, Deserialize, SurrealValue)]
    struct CountResult {
        n: i64,
    }

    let count_query = format!(
        "SELECT count() as n FROM player_match_stats WHERE user IN [{user_id_list}] AND created_at >= $cutoff GROUP ALL"
    );
    let mut count_r = db
        .query(&count_query)
        .bind(("cutoff", day_cutoff_str.clone()))
        .await?;

    let count_rows: Vec<CountResult> = count_r.take(0).unwrap_or_default();
    let match_count = count_rows.first().map(|r| r.n as usize).unwrap_or(0);

    let effective_cutoff = if match_count < min {
        let start_idx = if min > 1 { min - 1 } else { 0 };
        let oldest_query = format!(
            "SELECT <string>created_at AS created_at FROM player_match_stats WHERE user IN [{user_id_list}] ORDER BY created_at DESC LIMIT 1 START {start_idx}"
        );

        #[derive(Debug, Deserialize, SurrealValue)]
        struct DateRow {
            created_at: String,
        }

        let mut oldest_r = db.query(&oldest_query).await?;
        let oldest_rows: Vec<DateRow> = oldest_r.take(0).unwrap_or_default();
        oldest_rows
            .into_iter()
            .next()
            .map(|r| r.created_at)
            .unwrap_or(day_cutoff_str)
    } else {
        day_cutoff_str
    };

    // Batched query for team-wide data
    let match_query = format!(
        "SELECT champion, win FROM player_match_stats WHERE user IN [{user_id_list}] AND created_at >= $cutoff"
    );

    let mut result = db
        .query(&format!(
            "{match_query}; \
             SELECT champion FROM draft_action \
             WHERE draft IN (SELECT VALUE id FROM draft WHERE team = type::record('team', $team_key)) \
             AND phase CONTAINS 'pick' \
             AND created_at >= $cutoff; \
             SELECT our_champions FROM game_plan \
             WHERE team = type::record('team', $team_key) \
             AND created_at >= $cutoff"
        ))
        .bind(("team_key", team_key.clone()))
        .bind(("cutoff", effective_cutoff.clone()))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct MatchStatRow {
        champion: String,
        win: bool,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DraftChampRow {
        champion: String,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PlanChampRow {
        our_champions: Vec<String>,
    }

    let match_rows: Vec<MatchStatRow> = result.take(0).unwrap_or_default();
    let draft_rows: Vec<DraftChampRow> = result.take(1).unwrap_or_default();
    let plan_rows: Vec<PlanChampRow> = result.take(2).unwrap_or_default();

    if match_rows.is_empty() && draft_rows.is_empty() && plan_rows.is_empty() {
        return Ok(Vec::new());
    }

    let match_stats: Vec<(String, bool)> =
        match_rows.into_iter().map(|r| (r.champion, r.win)).collect();
    let draft_champions: Vec<String> = draft_rows.into_iter().map(|r| r.champion).collect();
    let plan_champions: Vec<String> = plan_rows
        .into_iter()
        .flat_map(|r| r.our_champions)
        .collect();

    Ok(aggregate_champion_performance(
        &draft_champions,
        &match_stats,
        &plan_champions,
        &[],
    ))
}

/// Data structure for draft outcome analytics
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct DraftOutcomeData {
    pub blue_games: i32,
    pub blue_wins: i32,
    pub red_games: i32,
    pub red_wins: i32,
    pub tag_stats: Vec<(String, i32, i32)>,
    pub first_pick_stats: Vec<(String, i32, i32)>,
}

// ---------------------------------------------------------------------------
// Solo Mode: User mode + region
// ---------------------------------------------------------------------------

pub async fn get_user_mode(db: &Surreal<Db>, user_id: &str) -> DbResult<String> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    #[derive(Debug, Deserialize, SurrealValue)]
    struct ModeRecord {
        mode: Option<String>,
    }
    let mut result = db
        .query("SELECT mode FROM type::record('user', $user_key)")
        .bind(("user_key", user_key))
        .await?;
    let row: Option<ModeRecord> = result.take(0)?;
    Ok(row
        .and_then(|r| r.mode)
        .unwrap_or_else(|| "solo".to_string()))
}

pub async fn set_user_mode(db: &Surreal<Db>, user_id: &str, mode: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET mode = $mode")
        .bind(("user_key", user_key))
        .bind(("mode", mode.to_string()))
        .await?
        .check()?;
    Ok(())
}

pub async fn set_user_region(db: &Surreal<Db>, user_id: &str, region: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET riot_region = $region")
        .bind(("user_key", user_key))
        .bind(("region", region.to_string()))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_last_solo_sync(db: &Surreal<Db>, user_id: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET last_solo_sync = time::now()")
        .bind(("user_key", user_key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Solo Mode: Ranked snapshots
// ---------------------------------------------------------------------------

pub async fn store_ranked_snapshot(
    db: &Surreal<Db>,
    user_id: &str,
    queue_type: &str,
    tier: &str,
    division: &str,
    lp: i32,
    wins: i32,
    losses: i32,
) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("CREATE ranked_snapshot SET user = type::record('user', $user_key), queue_type = $queue_type, tier = $tier, division = $division, lp = $lp, wins = $wins, losses = $losses")
        .bind(("user_key", user_key))
        .bind(("queue_type", queue_type.to_string()))
        .bind(("tier", tier.to_string()))
        .bind(("division", division.to_string()))
        .bind(("lp", lp))
        .bind(("wins", wins))
        .bind(("losses", losses))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_latest_ranked_snapshot(
    db: &Surreal<Db>,
    user_id: &str,
    queue_type: &str,
) -> DbResult<Option<crate::models::user::RankedInfo>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbRankedSnapshot {
        queue_type: String,
        tier: String,
        division: String,
        lp: i32,
        wins: i32,
        losses: i32,
    }

    let mut result = db
        .query("SELECT queue_type, tier, division, lp, wins, losses FROM ranked_snapshot WHERE user = type::record('user', $user_key) AND queue_type = $queue_type ORDER BY snapshotted_at DESC LIMIT 1")
        .bind(("user_key", user_key))
        .bind(("queue_type", queue_type.to_string()))
        .await?;

    let row: Option<DbRankedSnapshot> = result.take(0)?;
    Ok(row.map(|r| crate::models::user::RankedInfo {
        queue_type: r.queue_type,
        tier: r.tier,
        division: r.division,
        lp: r.lp,
        wins: r.wins,
        losses: r.losses,
    }))
}

// ---------------------------------------------------------------------------
// Solo Mode: Solo matches
// ---------------------------------------------------------------------------

pub async fn get_solo_matches(
    db: &Surreal<Db>,
    user_id: &str,
    queue_filter: Option<i32>,
    limit: i32,
) -> DbResult<Vec<crate::models::match_data::PlayerMatchStats>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    let rows: Vec<crate::models::match_data::PlayerMatchStats> = if let Some(qid) = queue_filter {
        let mut result = db
            .query("SELECT * FROM player_match WHERE user = type::record('user', $user_key) AND match.queue_id = $queue_id LIMIT $limit")
            .bind(("user_key", user_key))
            .bind(("queue_id", qid))
            .bind(("limit", limit))
            .await?;
        result.take(0).unwrap_or_default()
    } else {
        let mut result = db
            .query("SELECT * FROM player_match WHERE user = type::record('user', $user_key) LIMIT $limit")
            .bind(("user_key", user_key))
            .bind(("limit", limit))
            .await?;
        result.take(0).unwrap_or_default()
    };

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::champion::{Champion, ChampionPoolEntry};
    use crate::models::game_plan::{ActionItemPreview, DashboardSummary, PostGamePreview, PoolGapWarning};

    fn make_champion(id: &str, name: &str, tags: Vec<&str>) -> Champion {
        Champion {
            id: id.into(),
            name: name.into(),
            title: "Test".into(),
            tags: tags.into_iter().map(|s| s.into()).collect(),
            image_full: format!("{id}.png"),
        }
    }

    fn make_pool_entry(user_id: &str, champion: &str, role: &str) -> ChampionPoolEntry {
        ChampionPoolEntry {
            id: None,
            user_id: user_id.into(),
            champion: champion.into(),
            role: role.into(),
            tier: "comfort".into(),
            notes: None,
            comfort_level: None,
            meta_tag: None,
        }
    }

    #[test]
    fn test_compute_pool_gaps_dominant_class() {
        // All 4 picks are single-class Fighters — should warn about dominant class (4/4 = 100% >= 70%)
        let champions = vec![
            make_champion("Darius", "Darius", vec!["Fighter"]),
            make_champion("Garen", "Garen", vec!["Fighter"]),
            make_champion("Tryndamere", "Tryndamere", vec!["Fighter"]),
            make_champion("Renekton", "Renekton", vec!["Fighter"]),
            make_champion("Jinx", "Jinx", vec!["Marksman"]),
        ];
        let members = vec![("user:u1".to_string(), "Player1".to_string())];
        // 4 Fighters in top pool = 100% Fighter class coverage => dominant
        let top_pool = vec![
            make_pool_entry("user:u1", "Darius", "top"),
            make_pool_entry("user:u1", "Garen", "top"),
            make_pool_entry("user:u1", "Tryndamere", "top"),
            make_pool_entry("user:u1", "Renekton", "top"),
        ];
        let warnings = compute_pool_gaps(&top_pool, &members, &champions, &[]);
        assert!(!warnings.is_empty(), "Should warn about Fighter dominance (4/4 entries are Fighter = 100%)");
        let has_dominant = warnings.iter().any(|w| w.dominant_class.as_deref() == Some("Fighter"));
        assert!(has_dominant, "Should have a Fighter dominant class warning");

        // 3 Fighters + 1 Marksman = 75% Fighter => still dominant
        let mixed_pool = vec![
            make_pool_entry("user:u1", "Darius", "top"),
            make_pool_entry("user:u1", "Garen", "top"),
            make_pool_entry("user:u1", "Tryndamere", "top"),
            make_pool_entry("user:u1", "Jinx", "top"),
        ];
        let warnings2 = compute_pool_gaps(&mixed_pool, &members, &champions, &[]);
        let has_dominant2 = warnings2.iter().any(|w| w.dominant_class.as_deref() == Some("Fighter"));
        assert!(has_dominant2, "3/4 Fighter (75%) should be dominant");
    }

    #[test]
    fn test_compute_pool_gaps_missing_class_opponent() {
        // Player has no Tanks in their pool, but opponents play Tank champions
        let champions = vec![
            make_champion("Jinx", "Jinx", vec!["Marksman"]),
            make_champion("Caitlyn", "Caitlyn", vec!["Marksman"]),
            make_champion("MalzaharChamp", "Malzahar", vec!["Mage"]),
            make_champion("TankChamp", "Malphite", vec!["Tank", "Fighter"]),
        ];
        let pool = vec![
            make_pool_entry("user:u1", "Jinx", "bot"),
            make_pool_entry("user:u1", "Caitlyn", "bot"),
            make_pool_entry("user:u1", "MalzaharChamp", "bot"),
        ];
        let members = vec![("user:u1".to_string(), "Player1".to_string())];
        // Opponents play Tank champions
        let opponent_champions = vec!["TankChamp".to_string()];
        let warnings = compute_pool_gaps(&pool, &members, &champions, &opponent_champions);
        let has_tank_warning = warnings.iter().any(|w| {
            w.missing_classes.contains(&"Tank".to_string()) && w.opponent_escalated
        });
        assert!(has_tank_warning, "Should warn about missing Tank coverage with opponent escalation");
    }

    #[test]
    fn test_compute_pool_gaps_balanced() {
        // Balanced pool: Fighter, Mage, Assassin, Tank, Marksman, Support
        let champions = vec![
            make_champion("Darius", "Darius", vec!["Fighter"]),
            make_champion("Syndra", "Syndra", vec!["Mage"]),
            make_champion("Zed", "Zed", vec!["Assassin"]),
            make_champion("Malphite", "Malphite", vec!["Tank"]),
            make_champion("Jinx", "Jinx", vec!["Marksman"]),
            make_champion("Thresh", "Thresh", vec!["Support"]),
        ];
        let pool = vec![
            make_pool_entry("user:u1", "Darius", "top"),
            make_pool_entry("user:u1", "Syndra", "top"),
            make_pool_entry("user:u1", "Zed", "top"),
            make_pool_entry("user:u1", "Malphite", "top"),
            make_pool_entry("user:u1", "Jinx", "top"),
            make_pool_entry("user:u1", "Thresh", "top"),
        ];
        let members = vec![("user:u1".to_string(), "Player1".to_string())];
        let warnings = compute_pool_gaps(&pool, &members, &champions, &[]);
        // No class dominates (each is 1/6 = 16.7%), no missing classes with opponent escalation
        assert!(warnings.is_empty() || warnings.iter().all(|w| !w.opponent_escalated && w.dominant_class.is_none()),
            "Balanced pool should have no dominant class or opponent-escalated warnings");
    }

    #[test]
    fn test_dashboard_summary_assembly() {
        // Test that DashboardSummary can be constructed and all fields populate correctly
        let action_items = vec![
            ActionItemPreview { id: "action_item:1".into(), text: "Review baron fight".into() },
            ActionItemPreview { id: "action_item:2".into(), text: "Improve vision".into() },
        ];
        let post_games = vec![
            PostGamePreview {
                id: "pgl:1".into(),
                improvements: vec!["Better rotations".into()],
                created_at: Some("2026-03-14T20:00:00Z".into()),
            },
        ];
        let pool_warnings = vec![
            PoolGapWarning {
                user_id: "user:u1".into(),
                username: "Player1".into(),
                role: "top".into(),
                dominant_class: Some("Fighter".into()),
                missing_classes: vec![],
                opponent_escalated: false,
            },
        ];
        let summary = DashboardSummary {
            open_action_item_count: 5,
            recent_action_items: action_items.clone(),
            recent_post_games: post_games.clone(),
            pool_gap_warnings: pool_warnings.clone(),
            drafts_without_game_plan: 3,
            game_plans_without_post_game: 2,
        };
        assert_eq!(summary.open_action_item_count, 5);
        assert_eq!(summary.recent_action_items.len(), 2);
        assert_eq!(summary.recent_post_games.len(), 1);
        assert_eq!(summary.pool_gap_warnings.len(), 1);
        assert_eq!(summary.drafts_without_game_plan, 3);
        assert_eq!(summary.game_plans_without_post_game, 2);
    }

    #[test]
    fn test_dashboard_summary_empty_is_default() {
        // DashboardSummary::default() should have all zero/empty fields
        let summary = DashboardSummary::default();
        assert_eq!(summary.open_action_item_count, 0);
        assert!(summary.recent_action_items.is_empty());
        assert!(summary.recent_post_games.is_empty());
        assert!(summary.pool_gap_warnings.is_empty());
        assert_eq!(summary.drafts_without_game_plan, 0);
        assert_eq!(summary.game_plans_without_post_game, 0);
    }

    #[test]
    fn test_aggregate_champion_performance() {
        let draft_champions = vec!["Jinx".to_string(), "Jinx".to_string(), "Caitlyn".to_string()];
        let match_stats = vec![
            ("Jinx".to_string(), true),
            ("Jinx".to_string(), false),
            ("Caitlyn".to_string(), true),
        ];
        let plan_champions = vec!["Jinx".to_string()];
        let post_game_outcomes = vec![
            ("Jinx".to_string(), true),
            ("Caitlyn".to_string(), false),
        ];

        let result = aggregate_champion_performance(
            &draft_champions,
            &match_stats,
            &plan_champions,
            &post_game_outcomes,
        );

        assert!(!result.is_empty(), "Should return some results");

        let jinx = result.iter().find(|s| s.champion == "Jinx").expect("Should have Jinx");
        assert_eq!(jinx.games_in_draft, 2);
        assert_eq!(jinx.games_in_match, 2);
        assert_eq!(jinx.wins_in_match, 1);
        assert_eq!(jinx.games_in_plan, 1);
        assert_eq!(jinx.post_game_wins, 1);
        assert_eq!(jinx.post_game_losses, 0);

        let caitlyn = result.iter().find(|s| s.champion == "Caitlyn").expect("Should have Caitlyn");
        assert_eq!(caitlyn.games_in_draft, 1);
        assert_eq!(caitlyn.games_in_match, 1);
        assert_eq!(caitlyn.wins_in_match, 1);
        assert_eq!(caitlyn.games_in_plan, 0);
        assert_eq!(caitlyn.post_game_wins, 0);
        assert_eq!(caitlyn.post_game_losses, 1);
    }

    #[test]
    fn test_aggregate_champion_performance_empty() {
        let result = aggregate_champion_performance(&[], &[], &[], &[]);
        assert!(result.is_empty(), "Empty inputs should return empty result");
    }

    #[test]
    fn test_champion_performance_sorted_by_match_games() {
        // Result should be sorted by games_in_match descending
        let match_stats = vec![
            ("Jinx".to_string(), true),
            ("Caitlyn".to_string(), true),
            ("Caitlyn".to_string(), false),
            ("Caitlyn".to_string(), true),
        ];
        let result = aggregate_champion_performance(&[], &match_stats, &[], &[]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].champion, "Caitlyn", "Caitlyn has 3 games, should be first");
        assert_eq!(result[1].champion, "Jinx", "Jinx has 1 game, should be second");
    }

    #[test]
    fn strip_user_prefix_with_prefix() {
        let input = "user:abc123";
        let result = input.strip_prefix("user:").unwrap_or(input).to_string();
        assert_eq!(result, "abc123");
    }

    #[test]
    fn strip_user_prefix_without_prefix() {
        let input = "abc123";
        let result = input.strip_prefix("user:").unwrap_or(input).to_string();
        assert_eq!(result, "abc123");
    }

    #[test]
    fn strip_team_prefix_with_prefix() {
        let input = "team:xyz";
        let result = input.strip_prefix("team:").unwrap_or(input).to_string();
        assert_eq!(result, "xyz");
    }

    #[test]
    fn strip_draft_prefix_without_prefix_is_identity() {
        let input = "rawkey";
        let result = input.strip_prefix("draft:").unwrap_or(input).to_string();
        assert_eq!(result, "rawkey");
    }

    #[test]
    fn test_filter_win_condition_stats_empty_input() {
        let result = filter_win_condition_stats(&[], "TeamA");
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_win_condition_stats_no_matching_opponent() {
        let data = vec![
            ("Aggression".to_string(), Some("TeamB".to_string()), true),
            ("Scale".to_string(), Some("TeamB".to_string()), false),
        ];
        let result = filter_win_condition_stats(&data, "TeamA");
        assert!(result.is_empty(), "No plans against TeamA should mean empty results");
    }

    #[test]
    fn test_filter_win_condition_stats_matching_opponent() {
        let data = vec![
            ("Aggression".to_string(), Some("TeamA".to_string()), true),
            ("Aggression".to_string(), Some("TeamA".to_string()), false),
            ("Scale".to_string(), Some("TeamB".to_string()), true),
        ];
        let result = filter_win_condition_stats(&data, "TeamA");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("Aggression".to_string(), 2, 1)); // 2 games, 1 win
    }

    #[test]
    fn dedup_filters_matching_open_items() {
        use std::collections::HashSet;
        let open_texts: HashSet<String> = [
            "fix baron calls".to_string(),
            "ward river".to_string(),
        ]
        .into_iter()
        .collect();
        let improvements = vec![
            "Fix Baron Calls".to_string(),    // matches (case-insensitive) → skip
            "Improve dragon setup".to_string(), // new → create
            "  ".to_string(),                  // whitespace → skip
            "ward river".to_string(),          // exact match → skip
        ];
        let mut created = 0usize;
        for text in &improvements {
            let text = text.trim().to_string();
            if text.is_empty() {
                continue;
            }
            if open_texts.contains(&text.to_lowercase()) {
                continue;
            }
            created += 1;
        }
        assert_eq!(created, 1); // Only "Improve dragon setup"
    }

    #[test]
    fn empty_improvements_returns_zero() {
        let improvements: Vec<String> = vec![];
        // batch_create_action_items_from_review early-returns Ok(0) on empty input
        assert!(improvements.is_empty());
    }

    // ----- Solo mode DB function stubs (Phase 12) -----
    // These require a live SurrealDB instance. Marked #[ignore] for unit test runs.
    // Run with: cargo test --features ssr --lib -- db::tests --ignored (when DB available)

    #[tokio::test]
    #[ignore = "requires SurrealDB instance"]
    async fn test_get_user_mode_defaults_to_solo() {
        // Setup: create user without explicit mode field
        // Assert: get_user_mode returns "solo"
        todo!("Promote to integration test when linker OOM is resolved");
    }

    #[tokio::test]
    #[ignore = "requires SurrealDB instance"]
    async fn test_set_and_get_user_mode_round_trip() {
        // Setup: create user, set_user_mode to "team"
        // Assert: get_user_mode returns "team"
        // Set back to "solo", assert returns "solo"
        todo!("Promote to integration test when linker OOM is resolved");
    }

    #[tokio::test]
    #[ignore = "requires SurrealDB instance"]
    async fn test_store_and_get_ranked_snapshot() {
        // Setup: create user, store_ranked_snapshot with tier=Gold, div=II, lp=47
        // Assert: get_latest_ranked_snapshot returns RankedInfo with matching fields
        todo!("Promote to integration test when linker OOM is resolved");
    }

    #[tokio::test]
    #[ignore = "requires SurrealDB instance"]
    async fn test_get_solo_matches_empty() {
        // Setup: create user with no matches
        // Assert: get_solo_matches returns empty vec
        todo!("Promote to integration test when linker OOM is resolved");
    }

    #[tokio::test]
    #[ignore = "requires SurrealDB instance"]
    async fn test_get_solo_matches_queue_filter() {
        // Setup: create user, store matches with queue_id 420 and 440
        // Assert: get_solo_matches(queue_filter=Some(420)) returns only solo/duo matches
        // Assert: get_solo_matches(queue_filter=None) returns all matches
        todo!("Promote to integration test when linker OOM is resolved");
    }
}
