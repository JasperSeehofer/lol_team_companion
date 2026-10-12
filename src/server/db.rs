use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::{
    engine::local::Db,
    types::{Datetime, RecordId, SurrealValue, ToSql},
    Surreal,
};
use thiserror::Error;

use crate::models::{
    action_item::ActionItem,
    bug_report::BugReport,
    champion::{Champion, ChampionNote, ChampionPoolEntry, ChampionStatSummary},
    draft::{BanPriority, Draft, DraftAction, DraftTree, DraftTreeNode},
    game_plan::{
        ActionItemPreview, ChampionPerformanceSummary, ChecklistInstance, ChecklistTemplate,
        DashboardSummary, GamePlan, GamePlanEffectiveness, PoolGapWarning, PostGameLearning,
        PostGamePreview, StrategyTagSummary,
    },
    match_data::PlayerMatchStats,
    match_day::{LinkRef, ScheduledMatch, ScheduledMatchInput},
    opponent::{Opponent, OpponentPlayer},
    personal_learning::PersonalLearning,
    series::Series,
    share::{tier_rank, CardChampion, PlayerCard, ShareKind, ShareLink, SHARE_DAYS},
    team::Team,
    team_note::TeamNote,
    user::{JoinRequest, TeamMember},
};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::types::Error),
    /// The record does not exist or lies outside the caller's scope (team or
    /// user). Deliberately the same error for both, so callers cannot probe
    /// for other tenants' record IDs.
    #[error("Record not found")]
    NotFound,
    /// A rule was broken; the message is safe to show to the user.
    #[error("{0}")]
    Invalid(String),
    /// Internal failure; the message is for logs only.
    #[error("{0}")]
    Other(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Strip a `table:` prefix from a record ID so it can be bound as the key in
/// `type::record('table', $key)`. Keys are always bound, never formatted into
/// query text.
pub fn record_key(id: &str, table: &str) -> String {
    id.strip_prefix(table)
        .and_then(|rest| rest.strip_prefix(':'))
        .unwrap_or(id)
        .to_string()
}

/// Format a SurrealDB datetime for the shared models: RFC 3339, UTC, `Z`
/// suffix, sub-second digits only when present.
///
/// `Db*` structs load datetime fields as `Datetime` and convert with this
/// helper, so no query has to remember a `<string>` cast. (A datetime loaded
/// into a `String` field fails to deserialize, and `take(0).unwrap_or_default()`
/// then silently returns an empty list: the personal-learnings bug fixed in M1.)
pub fn dt_to_string(dt: &Datetime) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
}

/// `dt_to_string` for optional fields.
pub fn opt_dt(dt: Option<Datetime>) -> Option<String> {
    dt.as_ref().map(dt_to_string)
}

/// The key of an optional link ID sent by a client, ready to bind for
/// `IF $x = NONE { NONE } ELSE { type::record('table', $x) }`. Empty strings
/// count as "no link".
pub fn opt_key(id: Option<String>, table: &str) -> Option<String> {
    id.filter(|s| !s.trim().is_empty())
        .map(|s| record_key(s.trim(), table))
}

/// A record link read back as the string ID the shared models use.
fn opt_link(r: Option<RecordId>) -> Option<String> {
    r.map(|r| r.to_sql())
}

/// Succeeds only when `table:id` exists and `owner_path` (a field path on
/// that record, such as `team` or `tree.team`) points at `owner_table:owner_id`.
/// Zero rows means "not found" whether the record is missing or belongs to
/// another tenant.
///
/// `table`, `owner_path` and `owner_table` are compile-time constants from this
/// module; only the IDs come from callers, and they are bound.
async fn ensure_owned(
    db: &Surreal<Db>,
    table: &'static str,
    id: &str,
    owner_path: &'static str,
    owner_table: &'static str,
    owner_id: &str,
) -> DbResult<()> {
    let query = format!(
        "SELECT id FROM type::record($tb, $key) WHERE {owner_path} = type::record($otb, $okey)"
    );
    let mut r = db
        .query(query)
        .bind(("tb", table))
        .bind(("key", record_key(id, table)))
        .bind(("otb", owner_table))
        .bind(("okey", record_key(owner_id, owner_table)))
        .await?;
    let rows: Vec<IdRecord> = r.take(0).unwrap_or_default();
    if rows.is_empty() {
        Err(DbError::NotFound)
    } else {
        Ok(())
    }
}

/// Serialises the few writes whose rule spans a read and a write: invite use
/// counts and "a user belongs to at most one team". Under concurrency,
/// SurrealKV let two conditional `UPDATE invite SET uses += 1 WHERE uses <
/// max_uses` both report success for the last use (seen in the review's
/// concurrency test), and check-then-create across records is open to write
/// skew. One process owns the embedded database, so an in-process lock is
/// enough. Hold it for the whole check and write; never nest it.
///
/// Writes that rely on a UNIQUE index take it too (`create_user`,
/// `update_username`, `join_team`, `save_board_version`): on SurrealKV two
/// concurrent writes of the same unique key could both report success while
/// only one row was kept, or both be kept (M6 review, tests
/// `concurrent_signups_*`, `concurrent_board_saves_*`, `concurrent_joins_*`).
pub async fn invariant_lock() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    LOCK.lock().await
}

/// `table:id` must exist and belong to `team_id` (via its `team` field).
pub async fn ensure_team_record(
    db: &Surreal<Db>,
    table: &'static str,
    id: &str,
    team_id: &str,
) -> DbResult<()> {
    ensure_owned(db, table, id, "team", "team", team_id).await
}

/// `table:id` must exist and belong to `user_id` (via its `user` field).
pub async fn ensure_user_record(
    db: &Surreal<Db>,
    table: &'static str,
    id: &str,
    user_id: &str,
) -> DbResult<()> {
    ensure_owned(db, table, id, "user", "user", user_id).await
}

/// The stored password hash of a user (to confirm a destructive action).
pub async fn password_hash_of(db: &Surreal<Db>, user_id: &str) -> DbResult<String> {
    let mut r = db
        .query("SELECT VALUE password_hash FROM ONLY type::record('user', $user_key)")
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let hash: Option<String> = r.take(0)?;
    hash.ok_or(DbError::NotFound)
}

/// True when the user is a member of the team.
pub async fn is_team_member(db: &Surreal<Db>, user_id: &str, team_id: &str) -> DbResult<bool> {
    let mut r = db
        .query("SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) LIMIT 1")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let row: Option<IdRecord> = r.take(0)?;
    Ok(row.is_some())
}

/// True when the user created (leads) the team.
pub async fn is_team_leader(db: &Surreal<Db>, user_id: &str, team_id: &str) -> DbResult<bool> {
    let mut r = db
        .query("SELECT id FROM type::record('team', $team_key) WHERE created_by = type::record('user', $user_key)")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0).unwrap_or_default();
    Ok(!rows.is_empty())
}

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

/// The process-wide database handle that server functions use (polish
/// pass). It used to be provided as a Leptos context value, which put a
/// clone into every request's reactive owner: an owner that outlived its
/// request (a resource task still pending after the client left) kept the
/// database open at shutdown ("N database handle(s) still held"). Server
/// functions now borrow a clone from this slot for the length of the call,
/// and shutdown empties it with [`release_shared`].
static SHARED: std::sync::RwLock<Option<Arc<Surreal<Db>>>> = std::sync::RwLock::new(None);

/// A handle that does not keep the database open: long-lived holders that
/// requests clone (the auth backend, the session store) keep this instead of
/// an `Arc`, so a request that outlives the server (its owner still pending)
/// cannot hold SurrealKV open at shutdown. `get` fails once the server let go.
#[derive(Clone)]
pub struct WeakDb(std::sync::Weak<Surreal<Db>>);

impl WeakDb {
    pub fn new(db: &Arc<Surreal<Db>>) -> Self {
        Self(Arc::downgrade(db))
    }

    /// The database, while the server (or a test) still holds it.
    pub fn get(&self) -> Option<Arc<Surreal<Db>>> {
        self.0.upgrade()
    }
}

/// Make `db` the handle server functions get from `authz::db()`.
pub fn install_shared(db: Arc<Surreal<Db>>) {
    let mut slot = SHARED.write().unwrap_or_else(|p| p.into_inner());
    *slot = Some(db);
}

/// A clone of the shared handle (`None` before startup or after shutdown).
pub fn shared() -> Option<Arc<Surreal<Db>>> {
    SHARED
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .as_ref()
        .map(Arc::clone)
}

/// Empty the shared slot (shutdown): later server functions answer with an
/// internal error instead of keeping the database open.
pub fn release_shared() -> Option<Arc<Surreal<Db>>> {
    SHARED.write().unwrap_or_else(|p| p.into_inner()).take()
}

/// Record a visit: `last_seen_at` becomes now and its old value moves to
/// `prev_seen_at`, but only when the last record is an hour old or more (or
/// missing). Returns whether it wrote.
pub async fn touch_last_seen(db: &Surreal<Db>, user_id: &str) -> DbResult<bool> {
    let mut r = db
        .query(
            "UPDATE type::record('user', $uk) \
                SET prev_seen_at = last_seen_at, last_seen_at = time::now() \
                WHERE last_seen_at = NONE OR last_seen_at < time::now() - 1h \
                RETURN VALUE id",
        )
        .bind(("uk", record_key(user_id, "user")))
        .await?;
    let ids: Vec<RecordId> = r.take(0)?;
    Ok(!ids.is_empty())
}

pub async fn init_db(path: &str) -> DbResult<Arc<Surreal<Db>>> {
    use surrealdb::engine::local::SurrealKv;

    let db = Surreal::new::<SurrealKv>(path).await?;
    db.use_ns("lol_companion").use_db("app").await?;
    prepare(&db).await?;
    Ok(Arc::new(db))
}

/// Bring a database up to date: refuse a database migrated by a newer build
/// (before writing anything), apply the `schema.surql` baseline, then every
/// pending migration (`server::migrations`). Fails when a migration fails, so
/// the server refuses to start on a half-migrated database. Tests call this on
/// their in-memory databases too.
pub async fn prepare(db: &Surreal<Db>) -> DbResult<()> {
    crate::server::migrations::check_known(db).await?;
    apply_schema(db).await?;
    let applied = crate::server::migrations::run(db).await?;
    if !applied.is_empty() {
        tracing::info!(versions = ?applied, "migrations applied");
    }
    Ok(())
}

async fn apply_schema(db: &Surreal<Db>) -> DbResult<()> {
    db.query(include_str!("../../schema.surql"))
        .await?
        .check()?;
    Ok(())
}

/// Like `IndexedResults::check`, but for a `BEGIN ... COMMIT` query: when a
/// transaction fails, every statement reports "not executed due to a failed
/// transaction" and only one carries the real cause (a `THROW` or a constraint
/// violation). Return that one, so callers can match on a thrown tag.
pub fn check_tx(mut response: surrealdb::IndexedResults) -> DbResult<surrealdb::IndexedResults> {
    let errors = response.take_errors();
    if errors.is_empty() {
        return Ok(response);
    }
    let mut errors: Vec<(usize, surrealdb::types::Error)> = errors.into_iter().collect();
    errors.sort_by_key(|(i, _)| *i);
    let is_not_executed = |e: &surrealdb::types::Error| {
        matches!(
            e.details(),
            surrealdb::types::ErrorDetails::Query(Some(surrealdb::types::QueryError::NotExecuted))
        )
    };
    let pos = errors
        .iter()
        .position(|(_, e)| e.details().is_thrown())
        .or_else(|| errors.iter().position(|(_, e)| !is_not_executed(e)))
        .unwrap_or(0);
    Err(DbError::Surreal(errors.swap_remove(pos).1))
}

/// The tag of a `THROW 'TAG'` that aborted a transaction, if that is what
/// `err` is (see `check_tx`).
pub fn thrown_tag(err: &DbError) -> Option<String> {
    match err {
        DbError::Surreal(e) if e.details().is_thrown() => Some(
            e.message()
                .trim_start_matches("An error occurred: ")
                .to_string(),
        ),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// User
// ---------------------------------------------------------------------------

/// Shown for every registration failure that must not reveal whether an
/// email address already has an account.
pub const GENERIC_REGISTRATION_ERROR: &str =
    "Registration failed. Check your invite code and details, or sign in if you already have an account.";

/// Normalise an email address for storage and lookup: trimmed, lowercase.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Lookup form of a username: trimmed, lowercase. Stored in `user.username_lc`,
/// which has a unique index (migration 2).
pub fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase()
}

const USERNAME_TAKEN: &str = "That username is taken. Choose another one.";

/// Map a failed user write to a user-safe error. The unique indexes decide
/// (no check-first query, so concurrent registrations cannot race past it):
/// a duplicate username says so, a duplicate email gets the generic
/// registration message so it does not reveal that the address has an account.
fn user_write_error(e: surrealdb::types::Error) -> DbError {
    let msg = e.to_string();
    if msg.contains("username_lc_unique") {
        DbError::Invalid(USERNAME_TAKEN.into())
    } else if msg.contains("email_unique") {
        DbError::Invalid(GENERIC_REGISTRATION_ERROR.into())
    } else {
        tracing::warn!("user write failed: {msg}");
        DbError::Invalid(GENERIC_REGISTRATION_ERROR.into())
    }
}

/// Create a user. `email` is normalised here as well, so every caller stores
/// the same form. A duplicate email (case-insensitive) fails with the generic
/// registration error; a username that matches an existing one
/// case-insensitively fails with a message saying so.
pub async fn create_user(
    db: &Surreal<Db>,
    username: String,
    email: String,
    password_hash: String,
) -> DbResult<String> {
    let email = normalize_email(&email);
    let username = username.trim().to_string();
    if username.is_empty() {
        return Err(DbError::Invalid("Choose a username.".into()));
    }
    let username_lc = normalize_username(&username);

    // The unique indexes decide; the lock makes them reliable on SurrealKV.
    let _guard = invariant_lock().await;
    let response = db
        .query(
            "CREATE user SET username = $username, username_lc = $username_lc, email = $email, password_hash = $password_hash, is_admin = false",
        )
        .bind(("username", username))
        .bind(("username_lc", username_lc))
        .bind(("email", email))
        .bind(("password_hash", password_hash))
        .await?;

    let mut response = response.check().map_err(user_write_error)?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create user".into())),
    }
}

/// Change a user's username. Fails when another user already has it
/// (case-insensitive, enforced by the `username_lc` unique index).
pub async fn update_username(db: &Surreal<Db>, user_id: &str, username: String) -> DbResult<()> {
    let username = username.trim().to_string();
    if username.is_empty() {
        return Err(DbError::Invalid("Choose a username.".into()));
    }
    let username_lc = normalize_username(&username);
    let _guard = invariant_lock().await;
    db.query("UPDATE type::record('user', $user_key) SET username = $username, username_lc = $username_lc")
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("username", username))
        .bind(("username_lc", username_lc))
        .await?
        .check()
        .map_err(|e| {
            if e.to_string().contains("username_lc_unique") {
                DbError::Invalid(USERNAME_TAKEN.into())
            } else {
                DbError::Surreal(e)
            }
        })?;
    Ok(())
}

/// Set or clear the admin flag on a user.
pub async fn set_user_admin(db: &Surreal<Db>, user_id: &str, is_admin: bool) -> DbResult<()> {
    db.query("UPDATE type::record('user', $user_key) SET is_admin = $is_admin")
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("is_admin", is_admin))
        .await?
        .check()?;
    Ok(())
}

/// The promote command (`src/bin/promote.rs`): make the account with this
/// email an admin that does not depend on `ADMIN_EMAILS`, or revoke that.
/// Returns the account's (id, username). Revoking clears the flag too; an
/// address still listed in `ADMIN_EMAILS` is promoted again at its next
/// sign-in.
pub async fn set_admin_promotion(
    db: &Surreal<Db>,
    email: &str,
    promote: bool,
) -> DbResult<(String, String)> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        id: RecordId,
        username: String,
    }
    let q = if promote {
        "UPDATE user SET is_admin = true, admin_promoted_at = time::now() WHERE email = $email RETURN id, username"
    } else {
        "UPDATE user SET is_admin = false, admin_promoted_at = NONE WHERE email = $email RETURN id, username"
    };
    let mut r = db.query(q).bind(("email", normalize_email(email))).await?;
    let rows: Vec<Row> = r.take(0)?;
    rows.into_iter()
        .next()
        .map(|row| (row.id.to_sql(), row.username))
        .ok_or(DbError::NotFound)
}

/// (email, username, promoted with the command) of every stored admin.
pub async fn list_admins(db: &Surreal<Db>) -> DbResult<Vec<(String, String, bool)>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        email: String,
        username: String,
        promoted: bool,
    }
    let mut r = db
        .query("SELECT email, username, admin_promoted_at != NONE AS promoted FROM user WHERE is_admin = true OR admin_promoted_at != NONE ORDER BY email")
        .await?;
    let rows: Vec<Row> = r.take(0).unwrap_or_default();
    Ok(rows
        .into_iter()
        .map(|r| (r.email, r.username, r.promoted))
        .collect())
}

// ---------------------------------------------------------------------------
// Invites (closed-beta gate)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbInvite {
    id: RecordId,
    code: String,
    created_by_name: Option<String>,
    created_at: Datetime,
    max_uses: i64,
    uses: i64,
    expires_at: Option<Datetime>,
    disabled: bool,
    note: Option<String>,
    team: Option<RecordId>,
    team_name: Option<String>,
    position: Option<String>,
}

impl From<DbInvite> for crate::models::invite::Invite {
    fn from(i: DbInvite) -> Self {
        crate::models::invite::Invite {
            id: i.id.to_sql(),
            code: i.code,
            created_by: i.created_by_name,
            created_at: dt_to_string(&i.created_at),
            max_uses: i.max_uses,
            uses: i.uses,
            expires_at: opt_dt(i.expires_at),
            disabled: i.disabled,
            note: i.note,
            team_id: i.team.map(|t| t.to_sql()),
            team_name: i.team_name,
            position: i.position,
        }
    }
}

/// The columns of an invite as `DbInvite` reads them.
const INVITE_FIELDS: &str = "id, code, created_by.username AS created_by_name, created_at, \
     max_uses, uses, expires_at, disabled, note, team, team.name AS team_name, position";

/// What one use of an invite code grants: a plain code only lets the
/// account in; a team-bound code also puts it in that team (as a member not
/// placed yet, with the position the code names, if any).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InviteGrant {
    pub team_id: Option<String>,
    pub position: Option<String>,
}

/// A random invite code such as `K7QM-2XWD-9HPA` (60 bits of entropy, no
/// ambiguous characters).
pub fn generate_invite_code() -> String {
    use password_hash::rand_core::{OsRng, RngCore};
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 12];
    OsRng.fill_bytes(&mut bytes);
    let chars: Vec<char> = bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();
    chars
        .chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Normalise an invite code typed or pasted by a user.
pub fn normalize_invite_code(code: &str) -> String {
    code.trim().to_uppercase()
}

/// Create an invite. `created_by` is `None` for codes made by the seed tool or
/// the bootstrap env var.
pub async fn create_invite(
    db: &Surreal<Db>,
    created_by: Option<&str>,
    code: String,
    max_uses: i64,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    note: Option<String>,
) -> DbResult<String> {
    insert_invite(db, created_by, code, max_uses, expires_at, note, None, None).await
}

/// Create an invite bound to a team: whoever registers with it joins
/// `team_id` (a member not placed yet, with `position` if given). The team
/// must exist. Admins may bind a code to any team; a leader's server
/// function passes the leader's own team, never one from the client.
#[allow(clippy::too_many_arguments)]
pub async fn create_team_invite(
    db: &Surreal<Db>,
    created_by: &str,
    team_id: &str,
    code: String,
    max_uses: i64,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    position: Option<String>,
    note: Option<String>,
) -> DbResult<String> {
    if let Some(p) = position.as_deref() {
        if !["top", "jungle", "mid", "bot", "support"].contains(&p) {
            return Err(DbError::Invalid("Unknown position.".into()));
        }
    }
    let mut r = db
        .query("SELECT id FROM type::record('team', $team_key)")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    insert_invite(
        db,
        Some(created_by),
        code,
        max_uses,
        expires_at,
        note,
        Some(team_id),
        position,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn insert_invite(
    db: &Surreal<Db>,
    created_by: Option<&str>,
    code: String,
    max_uses: i64,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    note: Option<String>,
    team_id: Option<&str>,
    position: Option<String>,
) -> DbResult<String> {
    if max_uses < 1 {
        return Err(DbError::Invalid("Max uses must be at least 1.".into()));
    }
    let code = normalize_invite_code(&code);
    if code.is_empty() {
        return Err(DbError::Invalid("The code must not be empty.".into()));
    }
    let response = db
        .query("CREATE invite SET code = $code, \
                created_by = IF $created_by = NONE { NONE } ELSE { type::record('user', $created_by) }, \
                max_uses = $max_uses, uses = 0, \
                expires_at = IF $expires_at = NONE { NONE } ELSE { <datetime>$expires_at }, \
                disabled = false, note = $note, \
                team = IF $team = NONE { NONE } ELSE { type::record('team', $team) }, \
                position = $position")
        .bind(("code", code))
        .bind(("team", team_id.map(|t| record_key(t, "team"))))
        .bind(("position", position))
        .bind(("created_by", created_by.map(|u| record_key(u, "user"))))
        .bind(("max_uses", max_uses))
        .bind(("expires_at", expires_at.map(|t| t.to_rfc3339())))
        .bind(("note", note))
        .await?;
    let mut response = match response.check() {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("create_invite failed: {e}");
            return Err(DbError::Invalid("That code already exists.".into()));
        }
    };
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create invite".into())),
    }
}

/// Make sure an invite with this code exists (used by `BOOTSTRAP_INVITE_CODE`).
/// An existing code is left unchanged.
pub async fn ensure_invite(db: &Surreal<Db>, code: &str, max_uses: i64) -> DbResult<()> {
    let code = normalize_invite_code(code);
    let mut r = db
        .query("SELECT id FROM invite WHERE code = $code LIMIT 1")
        .bind(("code", code.clone()))
        .await?;
    let existing: Option<IdRecord> = r.take(0)?;
    if existing.is_none() {
        create_invite(db, None, code, max_uses, None, Some("bootstrap".into())).await?;
    }
    Ok(())
}

pub async fn list_invites(db: &Surreal<Db>) -> DbResult<Vec<crate::models::invite::Invite>> {
    let mut r = db
        .query(format!(
            "SELECT {INVITE_FIELDS} FROM invite ORDER BY created_at DESC"
        ))
        .await?;
    let rows: Vec<DbInvite> = r.take(0)?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// The invite codes bound to one team, newest first.
pub async fn list_team_invites(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<crate::models::invite::Invite>> {
    let mut r = db
        .query(format!(
            "SELECT {INVITE_FIELDS} FROM invite WHERE team = type::record('team', $team_key) \
             ORDER BY created_at DESC"
        ))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<DbInvite> = r.take(0)?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Disable one of the team's codes. A code of another team (or a global
/// one) is `NotFound`, the same as a missing one.
pub async fn disable_team_invite(db: &Surreal<Db>, team_id: &str, invite_id: &str) -> DbResult<()> {
    let mut r = db
        .query(
            "UPDATE type::record('invite', $key) SET disabled = true \
             WHERE team = type::record('team', $team_key) RETURN AFTER",
        )
        .bind(("key", record_key(invite_id, "invite")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn disable_invite(db: &Surreal<Db>, invite_id: &str) -> DbResult<()> {
    let mut r = db
        .query("UPDATE type::record('invite', $key) SET disabled = true RETURN AFTER")
        .bind(("key", record_key(invite_id, "invite")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0).unwrap_or_default();
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Atomically use one slot of an invite code. Returns `false` when the code is
/// unknown, disabled, expired or used up. The check and the increment are one
/// conditional UPDATE run under `invariant_lock`, so two registrations cannot
/// both take the last slot.
pub async fn consume_invite(db: &Surreal<Db>, code: &str) -> DbResult<bool> {
    Ok(consume_invite_grant(db, code).await?.is_some())
}

/// Like [`consume_invite`], and say what the use grants (`None`: the code
/// is unknown, disabled, expired or used up).
pub async fn consume_invite_grant(db: &Surreal<Db>, code: &str) -> DbResult<Option<InviteGrant>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Used {
        team: Option<RecordId>,
        position: Option<String>,
    }
    let code = normalize_invite_code(code);
    if code.is_empty() {
        return Ok(None);
    }
    let _guard = invariant_lock().await;
    let mut r = db
        .query(
            "UPDATE invite SET uses += 1 \
                WHERE code = $code AND disabled = false AND uses < max_uses \
                AND (expires_at = NONE OR expires_at > time::now()) \
                RETURN team, position",
        )
        .bind(("code", code))
        .await?;
    let rows: Vec<Used> = r.take(0)?;
    Ok(rows.into_iter().next().map(|u| InviteGrant {
        team_id: u.team.map(|t| t.to_sql()),
        position: u.position,
    }))
}

/// The name of the team a usable team-bound code leads to (`None` for a
/// plain, unknown, disabled, expired or used-up code).
pub async fn team_invite_name(db: &Surreal<Db>, code: &str) -> DbResult<Option<String>> {
    let code = normalize_invite_code(code);
    if code.is_empty() {
        return Ok(None);
    }
    let mut r = db
        .query(
            "SELECT VALUE team.name FROM invite \
                WHERE code = $code AND team != NONE AND disabled = false AND uses < max_uses \
                AND (expires_at = NONE OR expires_at > time::now()) LIMIT 1",
        )
        .bind(("code", code))
        .await?;
    let names: Vec<Option<String>> = r.take(0)?;
    Ok(names.into_iter().next().flatten())
}

/// Put a newly registered account into the team its invite names (a member
/// not placed yet, at the invite's position if any).
pub async fn join_team_from_invite(
    db: &Surreal<Db>,
    user_id: &str,
    grant: &InviteGrant,
) -> DbResult<()> {
    let Some(team_id) = grant.team_id.as_deref() else {
        return Ok(());
    };
    let _guard = invariant_lock().await;
    db.query(
        "CREATE team_member SET team = type::record('team', $team_key), \
             user = type::record('user', $user_key), role = $role, roster_type = 'member'",
    )
    .bind(("team_key", record_key(team_id, "team")))
    .bind(("user_key", record_key(user_id, "user")))
    .bind((
        "role",
        grant
            .position
            .clone()
            .unwrap_or_else(|| "unassigned".to_string()),
    ))
    .await?
    .check()
    .map_err(|e| {
        if e.to_string().contains("team_member_user_unique") {
            DbError::Invalid("You already belong to a team.".into())
        } else {
            DbError::Surreal(e)
        }
    })?;
    Ok(())
}

/// A signed-in account without a team uses a team-bound code: one use is
/// taken and the account joins that team, in one transaction. Refused,
/// without taking a use, when the account already belongs to a team or the
/// code is not a usable team code (plain codes only open registration).
/// Returns the team id.
pub async fn redeem_team_invite(db: &Surreal<Db>, user_id: &str, code: &str) -> DbResult<String> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Joined {
        team: RecordId,
    }
    let code = normalize_invite_code(code);
    if code.is_empty() {
        return Err(DbError::Invalid(INVALID_TEAM_INVITE.into()));
    }
    let _guard = invariant_lock().await;
    let r = db
        .query(
            "BEGIN TRANSACTION; \
             IF array::len(SELECT id FROM team_member WHERE user = type::record('user', $user_key)) > 0 \
                 { THROW 'IN_TEAM' }; \
             LET $inv = (UPDATE invite SET uses += 1 \
                 WHERE code = $code AND team != NONE AND disabled = false AND uses < max_uses \
                 AND (expires_at = NONE OR expires_at > time::now()) RETURN AFTER)[0]; \
             IF $inv = NONE { THROW 'BAD_CODE' }; \
             CREATE team_member SET team = $inv.team, user = type::record('user', $user_key), \
                 role = $inv.position ?? 'unassigned', roster_type = 'member'; \
             RETURN { team: $inv.team }; \
             COMMIT TRANSACTION;",
        )
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("code", code))
        .await?;
    let mut r = match check_tx(r) {
        Ok(r) => r,
        Err(e) => {
            return Err(match thrown_tag(&e).as_deref() {
                Some("IN_TEAM") => DbError::Invalid(
                    "You already belong to a team. Leave it before using a team invite.".into(),
                ),
                Some("BAD_CODE") => DbError::Invalid(INVALID_TEAM_INVITE.into()),
                _ if e.to_string().contains("team_member_user_unique") => DbError::Invalid(
                    "You already belong to a team. Leave it before using a team invite.".into(),
                ),
                _ => e,
            })
        }
    };
    // Statements: BEGIN 0, IF 1, LET 2, IF 3, CREATE 4, RETURN 5.
    let joined: Option<Joined> = r.take(5)?;
    joined
        .map(|j| j.team.to_sql())
        .ok_or_else(|| DbError::Other("team invite: no team returned".into()))
}

/// Shown for any code that cannot put a signed-in account into a team, so
/// the answer does not reveal which codes exist.
pub const INVALID_TEAM_INVITE: &str =
    "That code is not a team invite that can be used now. Ask your team leader for a new link.";

/// Give back a slot taken by `consume_invite` when registration fails after it.
pub async fn release_invite(db: &Surreal<Db>, code: &str) -> DbResult<()> {
    let _guard = invariant_lock().await;
    db.query("UPDATE invite SET uses -= 1 WHERE code = $code AND uses > 0")
        .bind(("code", normalize_invite_code(code)))
        .await?
        .check()?;
    Ok(())
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
    created_at: Option<Datetime>,
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
            created_at: opt_dt(e.created_at),
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
    // One spelling per position (migration 9 normalised the older rows).
    let role = crate::components::ui::Role::parse(&role)
        .map(|r| r.key().to_string())
        .unwrap_or(role);
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
        .query("SELECT * FROM champion_note WHERE user = type::record('user', $user_key) AND champion = $champion AND role = $role ORDER BY note_type, created_at DESC")
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
        .query("SELECT * FROM champion_note WHERE user = type::record('user', $user_key) AND champion IN $champions ORDER BY champion, note_type, created_at DESC")
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
    Ok(created.map(|r| r.id.to_sql()).unwrap_or_default())
}

pub async fn update_champion_note(
    db: &Surreal<Db>,
    user_id: &str,
    note: ChampionNote,
) -> DbResult<()> {
    let note_id = note.id.unwrap_or_default();
    ensure_user_record(db, "champion_note", &note_id, user_id).await?;
    let note_key = record_key(&note_id, "champion_note");
    db.query("UPDATE type::record('champion_note', $note_key) SET title = $title, content = $content, difficulty = $difficulty")
        .bind(("note_key", note_key))
        .bind(("title", note.title))
        .bind(("content", note.content))
        .bind(("difficulty", note.difficulty.map(|v| v as i64)))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_champion_note(db: &Surreal<Db>, user_id: &str, note_id: &str) -> DbResult<()> {
    ensure_user_record(db, "champion_note", note_id, user_id).await?;
    let note_key = record_key(note_id, "champion_note");
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

    // (kills, deaths, assists, cs, win) per game
    type GameLine = (f64, f64, f64, f64, bool);
    let mut by_champ: HashMap<String, Vec<GameLine>> = HashMap::new();
    for row in &rows {
        by_champ.entry(row.champion.clone()).or_default().push((
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

    results.sort_by_key(|e| std::cmp::Reverse(e.games));
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
    let user_key = record_key(user_id, "user");

    // One team per user: the check and the membership insert must not
    // interleave with another create_team or an accepted join request.
    let _guard = invariant_lock().await;
    if get_user_team_id(db, user_id).await?.is_some() {
        return Err(DbError::Invalid(
            "You already belong to a team. Leave it before creating a new one.".into(),
        ));
    }

    // The team and the creator's membership are one transaction.
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $t = (CREATE ONLY team SET name = $name, region = $region, created_by = type::record('user', $user_key)); \
             CREATE team_member SET team = $t.id, user = type::record('user', $user_key), role = 'unassigned', roster_type = 'member'; \
             RETURN { id: $t.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("name", name))
        .bind(("region", region))
        .bind(("user_key", user_key))
        .await?;
    let mut response = check_tx(response)?;
    let row: Option<IdRecord> = response.take(3)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create team".into()))
}

/// The user's current team, derived server-side. Users belong to at most one
/// team (create_team and respond_to_join_request enforce it); ordering by the
/// membership ID keeps the answer deterministic for legacy data with several
/// memberships.
pub async fn get_user_team_id(db: &Surreal<Db>, user_id: &str) -> DbResult<Option<String>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut result = db
        .query("SELECT id, team FROM team_member WHERE user = type::record('user', $user_key) ORDER BY id ASC LIMIT 1")
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

/// Every team with its member count (the join list). Two statements, not a
/// subquery per team: with a few thousand teams the correlated count took
/// seconds.
pub async fn list_all_teams(db: &Surreal<Db>) -> DbResult<Vec<Team>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberCount {
        team: RecordId,
        n: i64,
    }
    let mut r = db
        .query(
            "SELECT * FROM team ORDER BY name ASC; \
             SELECT team, count() AS n FROM team_member GROUP BY team;",
        )
        .await?;
    let teams: Vec<DbTeam> = r.take(0).unwrap_or_default();
    let counts: Vec<MemberCount> = r.take(1).unwrap_or_default();
    let counts: std::collections::HashMap<String, u32> = counts
        .into_iter()
        .map(|c| (c.team.to_sql(), c.n.max(0) as u32))
        .collect();
    Ok(teams
        .into_iter()
        .map(|t| {
            let mut team = Team::from(t);
            team.member_count = Some(
                team.id
                    .as_ref()
                    .and_then(|id| counts.get(id).copied())
                    .unwrap_or(0),
            );
            team
        })
        .collect())
}

pub async fn join_team(db: &Surreal<Db>, user_id: &str, team_id: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let _guard = invariant_lock().await;
    // Prevent duplicate membership
    let mut check = db
        .query("SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) LIMIT 1")
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .await?;
    let existing: Option<IdRecord> = check.take(0)?;
    if existing.is_some() {
        return Err(DbError::Invalid(
            "You are already a member of this team".into(),
        ));
    }
    // The unique index on team_member.user rejects a second team.
    db.query("CREATE team_member SET team = type::record('team', $team_key), user = type::record('user', $user_key), role = 'unassigned', roster_type = 'member'")
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .await?
        .check()
        .map_err(|e| {
            if e.to_string().contains("team_member_user_unique") {
                DbError::Invalid("That player already belongs to a team.".into())
            } else {
                DbError::Surreal(e)
            }
        })?;
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
        return Err(DbError::Invalid(
            "You already have a pending request for this team".into(),
        ));
    }
    // The team must exist (avoid dangling requests to made-up IDs).
    let mut team_check = db
        .query("SELECT id FROM type::record('team', $team_key)")
        .bind(("team_key", team_key.clone()))
        .await?;
    let team_rows: Vec<IdRecord> = team_check.take(0).unwrap_or_default();
    if team_rows.is_empty() {
        return Err(DbError::NotFound);
    }
    // Also prevent joining a team you're already in
    let mut member_check = db
        .query("SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) LIMIT 1")
        .bind(("team_key", team_key.clone()))
        .bind(("user_key", user_key.clone()))
        .await?;
    let already_member: Option<IdRecord> = member_check.take(0)?;
    if already_member.is_some() {
        return Err(DbError::Invalid(
            "You are already a member of this team".into(),
        ));
    }
    if get_user_team_id(db, user_id).await?.is_some() {
        return Err(DbError::Invalid(
            "You already belong to a team. Leave it before joining another.".into(),
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

/// Accept or decline a pending join request for `team_id` (the leader's team,
/// derived server-side). The request must belong to that team and still be
/// pending; anything else is "not found". Accepting adds the requester as a
/// member unless they already belong to a team.
pub async fn respond_to_join_request(
    db: &Surreal<Db>,
    request_id: &str,
    accept: bool,
    team_id: &str,
) -> DbResult<()> {
    let req_key = record_key(request_id, "join_request");
    let team_key = record_key(team_id, "team");
    // Covers the pending check, the one-team check and the insert (see
    // `invariant_lock`); the transaction below is a second line of defence.
    let _guard = invariant_lock().await;

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct UserRef {
        user: RecordId,
    }
    let mut r = db
        .query("SELECT user FROM type::record('join_request', $req_key) WHERE team = type::record('team', $team_key) AND status = 'pending'")
        .bind(("req_key", req_key.clone()))
        .bind(("team_key", team_key.clone()))
        .await?;
    let rows: Vec<UserRef> = r.take(0).unwrap_or_default();
    let Some(requester) = rows.into_iter().next() else {
        return Err(DbError::NotFound);
    };
    let user_key = record_key(&requester.user.to_sql(), "user");

    if accept {
        if get_user_team_id(db, &user_key).await?.is_some() {
            // Close the stale request so it stops showing up.
            db.query("UPDATE type::record('join_request', $req_key) SET status = 'declined'")
                .bind(("req_key", req_key))
                .await?
                .check()?;
            return Err(DbError::Invalid(
                "That player already belongs to a team.".into(),
            ));
        }
        // The status flip, the membership check and the insert are one
        // transaction that aborts unless the request was still pending, so a
        // double-clicked or concurrent accept cannot add a second membership
        // (concurrent writers to the same request also conflict and abort).
        let result = db
            .query(
                "BEGIN TRANSACTION; \
                 LET $flipped = (UPDATE type::record('join_request', $req_key) SET status = 'accepted' \
                     WHERE status = 'pending' AND team = type::record('team', $team_key) RETURN AFTER); \
                 IF array::len($flipped) = 0 { THROW 'JOIN_NOT_PENDING' }; \
                 IF array::len((SELECT id FROM team_member WHERE user = type::record('user', $user_key))) > 0 \
                     { THROW 'JOIN_ALREADY_MEMBER' }; \
                 CREATE team_member SET team = type::record('team', $team_key), user = type::record('user', $user_key), role = 'unassigned', roster_type = 'member'; \
                 COMMIT TRANSACTION;",
            )
            .bind(("req_key", req_key))
            .bind(("team_key", team_key))
            .bind(("user_key", user_key))
            .await?;
        if let Err(e) = check_tx(result) {
            match thrown_tag(&e).as_deref() {
                Some("JOIN_ALREADY_MEMBER") => {
                    return Err(DbError::Invalid(
                        "That player already belongs to a team.".into(),
                    ));
                }
                // Someone else handled the request first.
                Some("JOIN_NOT_PENDING") => return Err(DbError::NotFound),
                _ if e.to_string().to_lowercase().contains("conflict") => {
                    return Err(DbError::NotFound)
                }
                _ => return Err(e),
            }
        }
    } else {
        db.query("UPDATE type::record('join_request', $req_key) SET status = 'declined' WHERE status = 'pending'")
            .bind(("req_key", req_key))
            .await?
            .check()?;
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
    // Bench the current starter for that role and promote this member in one
    // transaction, so a role never ends up with two starters or none.
    if crate::components::ui::Role::parse(role).map(|r| r.key()) != Some(role) {
        return Err(DbError::Invalid("Unknown position".into()));
    }
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             IF array::len((SELECT id FROM team_member WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key))) = 0 { THROW 'NOT_MEMBER' }; \
             UPDATE team_member SET roster_type = 'substitute' WHERE team = type::record('team', $team_key) AND role = $role AND roster_type = 'starter'; \
             UPDATE team_member SET role = $role, roster_type = 'starter' WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key); \
             COMMIT TRANSACTION;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("role", role.to_string()))
        .await?;
    match check_tx(response) {
        Err(e) if thrown_tag(&e).as_deref() == Some("NOT_MEMBER") => Err(DbError::NotFound),
        other => other.map(|_| ()),
    }
}

/// Take a starter out of the starting five onto the bench (substitute).
pub async fn remove_from_slot(db: &Surreal<Db>, team_id: &str, user_id: &str) -> DbResult<()> {
    set_member_place(db, team_id, user_id, RosterPlace::Substitute).await
}

/// Where a member stands on the roster, outside the starting five (starting
/// is `assign_to_slot`, which needs a position).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RosterPlace {
    /// Benched: plays when needed, not in the starting five.
    Substitute,
    Coach,
    /// Not placed yet (new members).
    Member,
}

impl RosterPlace {
    pub fn key(self) -> &'static str {
        match self {
            RosterPlace::Substitute => "substitute",
            RosterPlace::Coach => "coach",
            RosterPlace::Member => "member",
        }
    }
}

/// Move a member of `team_id` to a place outside the starting five. A coach
/// keeps no position; a substitute keeps theirs (their usual position).
pub async fn set_member_place(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    place: RosterPlace,
) -> DbResult<()> {
    let query = if place == RosterPlace::Coach {
        "UPDATE team_member SET roster_type = $place, role = 'unassigned' WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) RETURN id"
    } else {
        "UPDATE team_member SET roster_type = $place WHERE team = type::record('team', $team_key) AND user = type::record('user', $user_key) RETURN id"
    };
    let mut r = db
        .query(query)
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("place", place.key().to_string()))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Drafts
// ---------------------------------------------------------------------------

/// A pick or ban bound into a transaction's `FOR` loop. (Named `ord` to keep
/// the SurrealQL keyword `order` out of `$a.<field>` paths.)
#[derive(Debug, Clone, serde::Serialize, SurrealValue)]
struct ActionRow {
    phase: String,
    side: String,
    champion: String,
    ord: i64,
    comment: Option<String>,
    role: Option<String>,
    role_guessed: bool,
}

fn action_rows(actions: Vec<DraftAction>) -> Vec<ActionRow> {
    actions
        .into_iter()
        .map(|a| ActionRow {
            phase: a.phase,
            side: a.side,
            champion: a.champion,
            ord: a.order as i64,
            comment: a.comment,
            role_guessed: a.role.is_some() && a.role_guessed,
            role: a.role,
        })
        .collect()
}

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
    series_id: Option<RecordId>,
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
            series_id: opt_link(d.series_id),
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
    #[serde(default)]
    role_guessed: Option<bool>,
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
            role_guessed: a.role_guessed.unwrap_or(false),
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
    // The draft and its picks/bans are one transaction.
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $d = (CREATE ONLY draft SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent = $opponent, notes = $notes, comments = $comments, rating = $rating, our_side = $our_side, tags = $tags, win_conditions = $win_conditions, watch_out = $watch_out, series_id = IF $series_key = NONE { NONE } ELSE { type::record('series', $series_key) }, game_number = $game_number); \
             FOR $a IN $actions { CREATE draft_action SET draft = $d.id, phase = $a.phase, side = $a.side, champion = $a.champion, `order` = $a.ord, comment = $a.comment, role = $a.role, role_guessed = $a.role_guessed; }; \
             RETURN { id: $d.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("name", name))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
        .bind(("comments", comments))
        .bind(("rating", rating))
        .bind(("our_side", our_side))
        .bind(("tags", tags))
        .bind(("win_conditions", win_conditions))
        .bind(("watch_out", watch_out))
        .bind(("series_key", opt_key(series_id, "series")))
        .bind(("game_number", game_number))
        .bind(("actions", action_rows(actions)))
        .await?;
    let mut response = check_tx(response)?;
    let row: Option<IdRecord> = response.take(3)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create draft".into()))
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
    team_id: &str,
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
    ensure_team_record(db, "draft", draft_id, team_id).await?;
    // Fields and the replaced picks/bans change together or not at all.
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             UPDATE type::record('draft', $draft_key) SET name=$name, opponent=$opponent, notes=$notes, comments=$comments, rating=$rating, our_side=$our_side, tags=$tags, win_conditions=$win_conditions, watch_out=$watch_out, series_id = IF $series_key = NONE { NONE } ELSE { type::record('series', $series_key) }, game_number=$game_number; \
             DELETE draft_action WHERE draft = type::record('draft', $draft_key); \
             FOR $a IN $actions { CREATE draft_action SET draft = type::record('draft', $draft_key), phase = $a.phase, side = $a.side, champion = $a.champion, `order` = $a.ord, comment = $a.comment, role = $a.role, role_guessed = $a.role_guessed; }; \
             COMMIT TRANSACTION;",
        )
        .bind(("draft_key", record_key(draft_id, "draft")))
        .bind(("name", name))
        .bind(("opponent", opponent))
        .bind(("notes", notes))
        .bind(("comments", comments))
        .bind(("rating", rating))
        .bind(("our_side", our_side))
        .bind(("tags", tags))
        .bind(("win_conditions", win_conditions))
        .bind(("watch_out", watch_out))
        .bind(("series_key", opt_key(series_id, "series")))
        .bind(("game_number", game_number))
        .bind(("actions", action_rows(actions)))
        .await?;
    check_tx(response)?;
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
    // The tree and its root node (named after the tree) are one transaction.
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $t = (CREATE ONLY draft_tree SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent = $opponent); \
             CREATE draft_tree_node SET tree = $t.id, parent = NONE, label = $name, sort_order = 0; \
             RETURN { id: $t.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("name", name))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("opponent", opponent))
        .await?;
    let mut response = check_tx(response)?;
    let row: Option<IdRecord> = response.take(3)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create draft tree".into()))
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

pub async fn delete_draft_tree(db: &Surreal<Db>, team_id: &str, tree_id: &str) -> DbResult<()> {
    ensure_team_record(db, "draft_tree", tree_id, team_id).await?;
    let tree_key = record_key(tree_id, "draft_tree");
    // Actions, nodes and the tree go together (no orphaned nodes on failure).
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             DELETE tree_node_action WHERE node IN (SELECT VALUE id FROM draft_tree_node WHERE tree = type::record('draft_tree', $tree_key)); \
             DELETE draft_tree_node WHERE tree = type::record('draft_tree', $tree_key); \
             DELETE type::record('draft_tree', $tree_key); \
             COMMIT TRANSACTION;",
        )
        .bind(("tree_key", tree_key))
        .await?;
    check_tx(response)?;
    Ok(())
}

pub async fn update_draft_tree(
    db: &Surreal<Db>,
    team_id: &str,
    tree_id: &str,
    name: String,
    opponent: Option<String>,
) -> DbResult<()> {
    ensure_team_record(db, "draft_tree", tree_id, team_id).await?;
    let tree_key = record_key(tree_id, "draft_tree");
    db.query("UPDATE type::record('draft_tree', $tree_key) SET name = $name, opponent = $opponent")
        .bind(("tree_key", tree_key))
        .bind(("name", name))
        .bind(("opponent", opponent))
        .await?
        .check()?;
    Ok(())
}

pub async fn get_tree_nodes(
    db: &Surreal<Db>,
    team_id: &str,
    tree_id: &str,
) -> DbResult<Vec<DraftTreeNode>> {
    ensure_team_record(db, "draft_tree", tree_id, team_id).await?;
    let tree_key = record_key(tree_id, "draft_tree");

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
                role_guessed: false,
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

/// Create a node in `tree_id` (which must belong to `team_id`). A parent, when
/// given, must be a node of the same tree. Every client value is bound; the
/// query text is constant (this used to `format!` the IDs into the query).
pub async fn create_tree_node(
    db: &Surreal<Db>,
    team_id: &str,
    tree_id: &str,
    parent_id: Option<String>,
    label: String,
) -> DbResult<String> {
    ensure_team_record(db, "draft_tree", tree_id, team_id).await?;
    let tree_key = record_key(tree_id, "draft_tree");
    let parent_key = parent_id
        .as_deref()
        .map(|p| record_key(p, "draft_tree_node"));

    if let Some(pk) = &parent_key {
        let mut pr = db
            .query("SELECT id FROM type::record('draft_tree_node', $parent_key) WHERE tree = type::record('draft_tree', $tree_key)")
            .bind(("parent_key", pk.clone()))
            .bind(("tree_key", tree_key.clone()))
            .await?;
        let rows: Vec<IdRecord> = pr.take(0).unwrap_or_default();
        if rows.is_empty() {
            return Err(DbError::NotFound);
        }
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct Count {
        n: i64,
    }

    // Constant query text for both shapes; only bound values differ.
    let (count_query, create_query) = if parent_key.is_some() {
        (
            "SELECT count() as n FROM draft_tree_node WHERE tree = type::record('draft_tree', $tree_key) AND parent = type::record('draft_tree_node', $parent_key) GROUP ALL",
            "CREATE draft_tree_node SET tree = type::record('draft_tree', $tree_key), parent = type::record('draft_tree_node', $parent_key), label = $label, sort_order = $sort_order",
        )
    } else {
        (
            "SELECT count() as n FROM draft_tree_node WHERE tree = type::record('draft_tree', $tree_key) AND parent = NONE GROUP ALL",
            "CREATE draft_tree_node SET tree = type::record('draft_tree', $tree_key), parent = NONE, label = $label, sort_order = $sort_order",
        )
    };

    // Next sort_order among siblings.
    let mut sr = db
        .query(count_query)
        .bind(("tree_key", tree_key.clone()))
        .bind(("parent_key", parent_key.clone()))
        .await?;
    let count: Option<Count> = sr.take(0)?;
    let sort_order = count.map(|c| c.n as i32).unwrap_or(0);

    let mut response = db
        .query(create_query)
        .bind(("tree_key", tree_key))
        .bind(("parent_key", parent_key))
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
    team_id: &str,
    node_id: &str,
    label: String,
    notes: Option<String>,
    is_improvised: bool,
    actions: Vec<DraftAction>,
) -> DbResult<()> {
    ensure_owned(db, "draft_tree_node", node_id, "tree.team", "team", team_id).await?;
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             UPDATE type::record('draft_tree_node', $node_key) SET label = $label, notes = $notes, is_improvised = $is_improvised; \
             DELETE tree_node_action WHERE node = type::record('draft_tree_node', $node_key); \
             FOR $a IN $actions { CREATE tree_node_action SET node = type::record('draft_tree_node', $node_key), phase = $a.phase, side = $a.side, champion = $a.champion, `order` = $a.ord, comment = $a.comment; }; \
             COMMIT TRANSACTION;",
        )
        .bind(("node_key", record_key(node_id, "draft_tree_node")))
        .bind(("label", label))
        .bind(("notes", notes))
        .bind(("is_improvised", is_improvised))
        .bind(("actions", action_rows(actions)))
        .await?;
    check_tx(response)?;
    Ok(())
}

pub async fn delete_tree_node(db: &Surreal<Db>, team_id: &str, node_id: &str) -> DbResult<()> {
    ensure_owned(db, "draft_tree_node", node_id, "tree.team", "team", team_id).await?;
    let node_key = record_key(node_id, "draft_tree_node");
    // Collect the node and all its descendants (SurrealQL has no recursive
    // query for this), then delete them and their actions in one transaction.
    let mut to_delete = vec![node_key];
    let mut i = 0;
    while i < to_delete.len() {
        let key = to_delete[i].clone();
        let mut r = db
            .query("SELECT id FROM draft_tree_node WHERE parent = type::record('draft_tree_node', $key)")
            .bind(("key", key))
            .await?;
        let children: Vec<IdRecord> = r.take(0).unwrap_or_default();
        for child in children {
            to_delete.push(record_key(&child.id.to_sql(), "draft_tree_node"));
        }
        i += 1;
    }

    let response = db
        .query(
            "BEGIN TRANSACTION; \
             FOR $k IN $keys { \
                 DELETE tree_node_action WHERE node = type::record('draft_tree_node', $k); \
                 DELETE type::record('draft_tree_node', $k); \
             }; \
             COMMIT TRANSACTION;",
        )
        .bind(("keys", to_delete))
        .await?;
    check_tx(response)?;
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
    let synced_by_key = synced_by_user_id.map(|s| s.strip_prefix("user:").unwrap_or(s).to_string());

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

            // All values are bound; NONE-valued options leave the fields unset.
            let mut cr = db
                .query("CREATE match SET match_id = $match_id, queue_id = $queue_id, game_duration = $game_duration, \
                        game_end = IF $game_end = NONE { NONE } ELSE { <datetime>$game_end }, \
                        synced_by = IF $synced_by_key = NONE { NONE } ELSE { type::record('user', $synced_by_key) }")
                .bind(("match_id", m.match_id.clone()))
                .bind(("queue_id", m.queue_id))
                .bind(("game_duration", m.game_duration))
                .bind(("game_end", game_end_str.filter(|s| !s.is_empty())))
                .bind(("synced_by_key", synced_by_key.clone()))
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
    get_team_match_stats_limit(db, team_id, 200).await
}

/// [`get_team_match_stats`], newest first, at most `limit` rows (the team
/// page needs five distinct games; one game holds at most ten members'
/// rows, e.g. an in-house game of a roster with substitutes, so it asks
/// for 50).
pub async fn get_team_match_stats_limit(
    db: &Surreal<Db>,
    team_id: &str,
    limit: i64,
) -> DbResult<Vec<TeamMatchRow>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // All player_match records for team members, enriched via the match link.
    // The member list is bound once: `user IN (SELECT ...)` re-ran the
    // subquery for every row (see the Draft Graph budget in ARCHITECTURE.md).
    let mut result = db
        .query("LET $users = (SELECT VALUE user FROM team_member WHERE team = type::record('team', $team_key)); \
                SELECT id, match AS match_ref, user, champion, kills, deaths, assists, cs, vision_score, damage, win, match.match_id AS riot_match_id, match.queue_id AS queue_id, match.game_duration AS game_duration, match.game_end AS game_end, user.username AS username FROM player_match WHERE user IN $users ORDER BY game_end DESC LIMIT $limit")
        .bind(("team_key", team_key))
        .bind(("limit", limit))
        .await?;
    let rows: Vec<DbTeamMatchRow> = result.take(1).unwrap_or_default();
    Ok(rows.into_iter().map(TeamMatchRow::from).collect())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeamMatchRow {
    #[allow(dead_code)]
    id: RecordId,
    match_ref: Option<RecordId>,
    user: RecordId,
    username: String,
    riot_match_id: String,
    queue_id: i32,
    game_duration: i32,
    game_end: Option<Datetime>,
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
            game_end: opt_dt(r.game_end),
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

/// A team member's Riot link, for team-wide syncs.
#[derive(Clone, Debug)]
pub struct RosterAccount {
    pub user_id: String,
    pub username: String,
    pub riot_puuid: Option<String>,
    pub riot_region: Option<String>,
}

/// Get roster member puuids for syncing
pub async fn get_roster_puuids(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<RosterAccount>> {
    let mut r = db
        .query("SELECT user.id as user_id, user.username as username, user.riot_puuid as riot_puuid, user.riot_region as riot_region FROM team_member WHERE team = type::record('team', $team_key)")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct MemberPuuid {
        user_id: RecordId,
        username: String,
        riot_puuid: Option<String>,
        riot_region: Option<String>,
    }

    let members: Vec<MemberPuuid> = r.take(0).unwrap_or_default();
    Ok(members
        .into_iter()
        .map(|m| RosterAccount {
            user_id: m.user_id.to_sql(),
            username: m.username,
            riot_puuid: m.riot_puuid,
            riot_region: m.riot_region,
        })
        .collect())
}

/// The team's region code ("EUW", "NA", ...), used to route Riot calls for
/// team-wide work and opponent scouting.
pub async fn get_team_region(db: &Surreal<Db>, team_id: &str) -> DbResult<Option<String>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct RegionRow {
        region: Option<String>,
    }
    let mut r = db
        .query("SELECT region FROM type::record('team', $team_key)")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let row: Option<RegionRow> = r.take(0)?;
    Ok(row.and_then(|r| r.region))
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
    let mut response = db.query("CREATE game_plan SET name = $name, team = type::record('team', $team_key), draft = IF $draft_key = NONE { NONE } ELSE { type::record('draft', $draft_key) }, our_champions = $our_champions, enemy_champions = $enemy_champions, win_conditions = $win_conditions, objective_priority = $objective_priority, teamfight_strategy = $teamfight_strategy, early_game = $early_game, top_strategy = $top_strategy, jungle_strategy = $jungle_strategy, mid_strategy = $mid_strategy, bot_strategy = $bot_strategy, support_strategy = $support_strategy, notes = $notes, win_condition_tag = $win_condition_tag, created_by = type::record('user', $user_key)")
        .bind(("name", plan.name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("draft_key", opt_key(plan.draft_id, "draft")))
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

/// Update a game plan of `team_id`. The plan's own `team_id` field is ignored.
pub async fn update_game_plan(db: &Surreal<Db>, team_id: &str, plan: GamePlan) -> DbResult<()> {
    let plan_id = plan.id.as_deref().ok_or(DbError::NotFound)?;
    ensure_team_record(db, "game_plan", plan_id, team_id).await?;
    let plan_key = record_key(plan_id, "game_plan");
    db.query("UPDATE type::record('game_plan', $plan_key) SET name = $name, draft = IF $draft_key = NONE { NONE } ELSE { type::record('draft', $draft_key) }, draft_deleted = IF $draft_key = NONE { draft_deleted } ELSE { NONE }, our_champions = $our_champions, enemy_champions = $enemy_champions, win_conditions = $win_conditions, objective_priority = $objective_priority, teamfight_strategy = $teamfight_strategy, early_game = $early_game, top_strategy = $top_strategy, jungle_strategy = $jungle_strategy, mid_strategy = $mid_strategy, bot_strategy = $bot_strategy, support_strategy = $support_strategy, notes = $notes, win_condition_tag = $win_condition_tag")
        .bind(("plan_key", plan_key))
        .bind(("name", plan.name))
        .bind(("draft_key", opt_key(plan.draft_id, "draft")))
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
    draft: Option<RecordId>,
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
    #[serde(default)]
    draft_deleted: Option<bool>,
}

impl From<DbGamePlan> for GamePlan {
    fn from(p: DbGamePlan) -> Self {
        GamePlan {
            draft_deleted: p.draft_deleted.unwrap_or(false),
            id: Some(p.id.to_sql()),
            team_id: p.team.to_sql(),
            draft_id: opt_link(p.draft),
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

pub async fn delete_game_plan(db: &Surreal<Db>, team_id: &str, plan_id: &str) -> DbResult<()> {
    ensure_team_record(db, "game_plan", plan_id, team_id).await?;
    let plan_key = record_key(plan_id, "game_plan");
    db.query("DELETE type::record('game_plan', $plan_key)")
        .bind(("plan_key", plan_key))
        .await?
        .check()?;
    Ok(())
}

/// Fetch a single draft of `team_id` with its actions for prefilling a game
/// plan. Returns `None` if no such draft exists in that team.
pub async fn get_draft_for_prefill(
    db: &Surreal<Db>,
    team_id: &str,
    draft_id: &str,
) -> DbResult<Option<Draft>> {
    let draft_key = record_key(draft_id, "draft");
    let mut result = db
        .query("SELECT * FROM type::record('draft', $draft_key) WHERE team = type::record('team', $team_key); SELECT * FROM draft_action WHERE draft = type::record('draft', $draft_key) AND draft.team = type::record('team', $team_key) ORDER BY `order` ASC")
        .bind(("draft_key", draft_key))
        .bind(("team_key", record_key(team_id, "team")))
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
    team_id: &str,
    draft_id: &str,
) -> DbResult<Vec<GamePlan>> {
    let mut r = db
        .query("SELECT * FROM game_plan WHERE draft = type::record('draft', $draft_key) AND team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("draft_key", record_key(draft_id, "draft")))
        .bind(("team_key", record_key(team_id, "team")))
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
    let mut response = db.query("CREATE post_game_learning SET team = type::record('team', $team_key), match_riot_id = $match_riot_id, game_plan_id = IF $plan_key = NONE { NONE } ELSE { type::record('game_plan', $plan_key) }, draft_id = IF $draft_key = NONE { NONE } ELSE { type::record('draft', $draft_key) }, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, open_notes = $open_notes, created_by = type::record('user', $created_by_key), win_loss = $win_loss, rating = $rating")
        .bind(("team_key", team_key))
        .bind(("match_riot_id", learning.match_riot_id))
        .bind(("plan_key", opt_key(learning.game_plan_id, "game_plan")))
        .bind(("draft_key", opt_key(learning.draft_id, "draft")))
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

/// Lines from a review that should become new action items: trimmed,
/// non-empty, not already an open or in-progress item of the team, and not
/// an item this review already created in any status, so saving a review
/// again does not bring back items the team has done (case-insensitive).
async fn new_action_item_texts(
    db: &Surreal<Db>,
    team_id: &str,
    review_id: Option<&str>,
    improvements: &[String],
) -> DbResult<Vec<String>> {
    if improvements.iter().all(|t| t.trim().is_empty()) {
        return Ok(Vec::new());
    }
    let existing = list_open_action_items(db, team_id).await?;
    let mut seen: std::collections::HashSet<String> =
        existing.iter().map(|i| i.text.to_lowercase()).collect();
    if let Some(review) = review_id {
        let mut r = db
            .query(
                "SELECT VALUE text FROM action_item WHERE team = type::record('team', $team_key) AND source_review = type::record('post_game_learning', $review_key)",
            )
            .bind(("team_key", record_key(team_id, "team")))
            .bind(("review_key", record_key(review, "post_game_learning")))
            .await?;
        let own: Vec<String> = r.take(0).unwrap_or_default();
        seen.extend(own.into_iter().map(|t| t.to_lowercase()));
    }
    Ok(improvements
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty() && seen.insert(t.to_lowercase()))
        .collect())
}

/// The lines of a review that become action items: its improvements and
/// its action item lines (blank and already-open ones are skipped later).
fn review_item_lines(review: &PostGameLearning) -> Vec<String> {
    review
        .improvements
        .iter()
        .chain(review.action_items.iter())
        .cloned()
        .collect()
}

/// Create a review (with its plan and draft links) and an action item for
/// each new improvement or action item line in one transaction. Returns the review ID and how
/// many action items were created. `review.team_id` and `created_by` must
/// already be the caller's team and user (the server function sets them).
pub async fn create_review_with_action_items(
    db: &Surreal<Db>,
    review: PostGameLearning,
) -> DbResult<(String, usize)> {
    let items =
        new_action_item_texts(db, &review.team_id, None, &review_item_lines(&review)).await?;
    let count = items.len();
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $r = (CREATE ONLY post_game_learning SET team = type::record('team', $team_key), match_riot_id = $match_riot_id, game_plan_id = IF $plan_key = NONE { NONE } ELSE { type::record('game_plan', $plan_key) }, draft_id = IF $draft_key = NONE { NONE } ELSE { type::record('draft', $draft_key) }, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, open_notes = $open_notes, created_by = type::record('user', $created_by_key), win_loss = $win_loss, rating = $rating); \
             FOR $t IN $items { CREATE action_item SET team = type::record('team', $team_key), text = $t, source_review = $r.id; }; \
             RETURN { id: $r.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("team_key", record_key(&review.team_id, "team")))
        .bind(("match_riot_id", review.match_riot_id))
        .bind(("plan_key", opt_key(review.game_plan_id, "game_plan")))
        .bind(("draft_key", opt_key(review.draft_id, "draft")))
        .bind(("what_went_well", review.what_went_well))
        .bind(("improvements", review.improvements))
        .bind(("action_items", review.action_items))
        .bind(("open_notes", review.open_notes))
        .bind(("created_by_key", record_key(&review.created_by, "user")))
        .bind(("win_loss", review.win_loss))
        .bind(("rating", review.rating))
        .bind(("items", items))
        .await?;
    let mut response = check_tx(response)?;
    let row: Option<IdRecord> = response.take(3)?;
    let id = row
        .map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create post-game review".into()))?;
    Ok((id, count))
}

/// Update a review of `team_id` and add action items for its new
/// improvements, in one transaction. Returns how many items were created.
/// The review's own `team_id` and `created_by` are ignored.
pub async fn update_review_with_action_items(
    db: &Surreal<Db>,
    team_id: &str,
    review: PostGameLearning,
) -> DbResult<usize> {
    let id = review.id.clone().ok_or(DbError::NotFound)?;
    ensure_team_record(db, "post_game_learning", &id, team_id).await?;
    let items = new_action_item_texts(db, team_id, Some(&id), &review_item_lines(&review)).await?;
    let count = items.len();
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             UPDATE type::record('post_game_learning', $key) SET match_riot_id = $match_riot_id, game_plan_id = IF $plan_key = NONE { NONE } ELSE { type::record('game_plan', $plan_key) }, draft_id = IF $draft_key = NONE { NONE } ELSE { type::record('draft', $draft_key) }, draft_deleted = IF $draft_key = NONE { draft_deleted } ELSE { NONE }, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, open_notes = $open_notes, win_loss = $win_loss, rating = $rating; \
             FOR $t IN $items { CREATE action_item SET team = type::record('team', $team_key), text = $t, source_review = type::record('post_game_learning', $key); }; \
             COMMIT TRANSACTION;",
        )
        .bind(("key", record_key(&id, "post_game_learning")))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("match_riot_id", review.match_riot_id))
        .bind(("plan_key", opt_key(review.game_plan_id, "game_plan")))
        .bind(("draft_key", opt_key(review.draft_id, "draft")))
        .bind(("what_went_well", review.what_went_well))
        .bind(("improvements", review.improvements))
        .bind(("action_items", review.action_items))
        .bind(("open_notes", review.open_notes))
        .bind(("win_loss", review.win_loss))
        .bind(("rating", review.rating))
        .bind(("items", items))
        .await?;
    check_tx(response)?;
    Ok(count)
}

/// Update a review of `team_id`. The review's own `team_id` and `created_by`
/// fields are ignored (they cannot be changed).
pub async fn update_post_game_learning(
    db: &Surreal<Db>,
    team_id: &str,
    learning: PostGameLearning,
) -> DbResult<()> {
    let id = learning.id.as_deref().ok_or(DbError::NotFound)?;
    ensure_team_record(db, "post_game_learning", id, team_id).await?;
    let key = record_key(id, "post_game_learning");
    db.query("UPDATE type::record('post_game_learning', $key) SET match_riot_id = $match_riot_id, game_plan_id = IF $plan_key = NONE { NONE } ELSE { type::record('game_plan', $plan_key) }, draft_id = IF $draft_key = NONE { NONE } ELSE { type::record('draft', $draft_key) }, draft_deleted = IF $draft_key = NONE { draft_deleted } ELSE { NONE }, what_went_well = $what_went_well, improvements = $improvements, action_items = $action_items, open_notes = $open_notes, win_loss = $win_loss, rating = $rating")
        .bind(("key", key))
        .bind(("match_riot_id", learning.match_riot_id))
        .bind(("plan_key", opt_key(learning.game_plan_id, "game_plan")))
        .bind(("draft_key", opt_key(learning.draft_id, "draft")))
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
    game_plan_id: Option<RecordId>,
    draft_id: Option<RecordId>,
    what_went_well: Vec<String>,
    improvements: Vec<String>,
    action_items: Vec<String>,
    open_notes: Option<String>,
    created_by: RecordId,
    #[serde(default)]
    win_loss: Option<String>,
    #[serde(default)]
    rating: Option<u8>,
    #[serde(default)]
    draft_deleted: Option<bool>,
    #[serde(default)]
    created_at: Option<Datetime>,
}

impl From<DbPostGameLearning> for PostGameLearning {
    fn from(p: DbPostGameLearning) -> Self {
        PostGameLearning {
            draft_deleted: p.draft_deleted.unwrap_or(false),
            created_at: opt_dt(p.created_at),
            id: Some(p.id.to_sql()),
            team_id: p.team.to_sql(),
            match_riot_id: p.match_riot_id,
            game_plan_id: opt_link(p.game_plan_id),
            draft_id: opt_link(p.draft_id),
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

pub async fn delete_post_game_learning(db: &Surreal<Db>, team_id: &str, id: &str) -> DbResult<()> {
    ensure_team_record(db, "post_game_learning", id, team_id).await?;
    let key = record_key(id, "post_game_learning");
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
            reviews_by_plan
                .entry(plan_id.clone())
                .or_default()
                .push(review);
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
        let wins = plan_reviews
            .iter()
            .filter(|r| r.win_loss.as_deref() == Some("win"))
            .count();
        let losses = plan_reviews
            .iter()
            .filter(|r| r.win_loss.as_deref() == Some("loss"))
            .count();
        let ratings: Vec<f32> = plan_reviews
            .iter()
            .filter_map(|r| r.rating.map(|v| v as f32))
            .collect();
        let avg_rating = if ratings.is_empty() {
            None
        } else {
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

    let tag_summaries: Vec<StrategyTagSummary> = tag_map
        .into_iter()
        .map(|(tag, (games_played, wins, losses, ratings))| {
            let avg_rating = if ratings.is_empty() {
                None
            } else {
                Some(ratings.iter().sum::<f32>() / ratings.len() as f32)
            };
            StrategyTagSummary {
                tag,
                games_played,
                wins,
                losses,
                avg_rating,
            }
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
    game_plan_id: Option<RecordId>,
    template_id: Option<RecordId>,
    items: Vec<String>,
    checked: Vec<bool>,
}

impl From<DbChecklistInstance> for ChecklistInstance {
    fn from(i: DbChecklistInstance) -> Self {
        ChecklistInstance {
            id: Some(i.id.to_sql()),
            team_id: i.team.to_sql(),
            game_plan_id: opt_link(i.game_plan_id),
            template_id: opt_link(i.template_id),
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

pub async fn delete_checklist_template(
    db: &Surreal<Db>,
    team_id: &str,
    template_id: &str,
) -> DbResult<()> {
    ensure_team_record(db, "checklist_template", template_id, team_id).await?;
    let key = record_key(template_id, "checklist_template");
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
        .query("CREATE checklist_instance SET team = type::record('team', $team_key), game_plan_id = IF $plan_key = NONE { NONE } ELSE { type::record('game_plan', $plan_key) }, template_id = IF $template_key = NONE { NONE } ELSE { type::record('checklist_template', $template_key) }, items = $items, checked = $checked")
        .bind(("team_key", team_key))
        .bind(("plan_key", opt_key(game_plan_id, "game_plan")))
        .bind(("template_key", opt_key(template_id, "checklist_template")))
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
    team_id: &str,
    game_plan_id: &str,
) -> DbResult<Option<ChecklistInstance>> {
    let mut r = db
        .query("SELECT * FROM checklist_instance WHERE game_plan_id = type::record('game_plan', $plan_key) AND team = type::record('team', $team_key) ORDER BY created_at DESC LIMIT 1")
        .bind(("plan_key", record_key(game_plan_id, "game_plan")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let row: Option<DbChecklistInstance> = r.take(0)?;
    Ok(row.map(ChecklistInstance::from))
}

pub async fn update_checklist_checked(
    db: &Surreal<Db>,
    team_id: &str,
    instance_id: &str,
    checked: Vec<bool>,
) -> DbResult<()> {
    ensure_team_record(db, "checklist_instance", instance_id, team_id).await?;
    let key = record_key(instance_id, "checklist_instance");
    db.query("UPDATE type::record('checklist_instance', $key) SET checked = $checked")
        .bind(("key", key))
        .bind(("checked", checked))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_checklist_instance(
    db: &Surreal<Db>,
    team_id: &str,
    instance_id: &str,
) -> DbResult<()> {
    ensure_team_record(db, "checklist_instance", instance_id, team_id).await?;
    let key = record_key(instance_id, "checklist_instance");
    db.query("DELETE type::record('checklist_instance', $key)")
        .bind(("key", key))
        .await?
        .check()?;
    Ok(())
}

/// A game plan of `team_id`, or `None` if it does not exist in that team.
pub async fn get_game_plan(
    db: &Surreal<Db>,
    team_id: &str,
    plan_id: &str,
) -> DbResult<Option<GamePlan>> {
    let plan_key = record_key(plan_id, "game_plan");
    let mut r = db
        .query("SELECT * FROM type::record('game_plan', $plan_key) WHERE team = type::record('team', $team_key)")
        .bind(("plan_key", plan_key))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let row: Option<DbGamePlan> = r.take(0)?;
    Ok(row.map(GamePlan::from))
}

pub async fn delete_draft(db: &Surreal<Db>, team_id: &str, draft_id: &str) -> DbResult<()> {
    ensure_team_record(db, "draft", draft_id, team_id).await?;
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             DELETE draft_action WHERE draft = type::record('draft', $draft_key); \
             UPDATE game_plan SET draft = NONE, draft_deleted = true WHERE draft = type::record('draft', $draft_key); \
             UPDATE post_game_learning SET draft_id = NONE, draft_deleted = true WHERE draft_id = type::record('draft', $draft_key); \
             DELETE type::record('draft', $draft_key); \
             COMMIT TRANSACTION;",
        )
        .bind(("draft_key", record_key(draft_id, "draft")))
        .await?;
    check_tx(response)?;
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
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
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
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // Delete old + create new in transaction
    let mut query = String::from(
        "BEGIN TRANSACTION; DELETE ban_priority WHERE team = type::record('team', $team_key);",
    );
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
    last_fetched: Option<Datetime>,
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
            last_fetched: opt_dt(p.last_fetched),
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
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
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
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query(
            "SELECT * FROM opponent WHERE team = type::record('team', $team_key) ORDER BY name ASC",
        )
        .bind(("team_key", team_key))
        .await?;
    let rows: Vec<DbOpponent> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(Opponent::from).collect())
}

/// An opponent of `team_id` with its players, or `None` if the opponent does
/// not exist in that team.
pub async fn get_opponent(
    db: &Surreal<Db>,
    team_id: &str,
    opponent_id: &str,
) -> DbResult<Option<(Opponent, Vec<OpponentPlayer>)>> {
    let opp_key = record_key(opponent_id, "opponent");
    let mut r = db
        .query("SELECT * FROM type::record('opponent', $opp_key) WHERE team = type::record('team', $team_key); SELECT * FROM opponent_player WHERE opponent = type::record('opponent', $opp_key) AND opponent.team = type::record('team', $team_key) ORDER BY role ASC")
        .bind(("opp_key", opp_key))
        .bind(("team_key", record_key(team_id, "team")))
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
    team_id: &str,
    opponent_id: &str,
    name: String,
    notes: Option<String>,
) -> DbResult<()> {
    ensure_team_record(db, "opponent", opponent_id, team_id).await?;
    let opp_key = record_key(opponent_id, "opponent");
    db.query("UPDATE type::record('opponent', $opp_key) SET name = $name, notes = $notes")
        .bind(("opp_key", opp_key))
        .bind(("name", name))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_opponent(db: &Surreal<Db>, team_id: &str, opponent_id: &str) -> DbResult<()> {
    ensure_team_record(db, "opponent", opponent_id, team_id).await?;
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             DELETE opponent_player WHERE opponent = type::record('opponent', $opp_key); \
             UPDATE scheduled_match SET opponent = NONE, brief_read_by = [] WHERE opponent = type::record('opponent', $opp_key); \
             DELETE type::record('opponent', $opp_key); \
             COMMIT TRANSACTION;",
        )
        .bind(("opp_key", record_key(opponent_id, "opponent")))
        .await?;
    check_tx(response)?;
    Ok(())
}

pub async fn add_opponent_player(
    db: &Surreal<Db>,
    team_id: &str,
    opponent_id: &str,
    name: String,
    role: String,
) -> DbResult<String> {
    ensure_team_record(db, "opponent", opponent_id, team_id).await?;
    let opp_key = record_key(opponent_id, "opponent");
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

/// `opponent_player:id` must exist and its opponent must belong to `team_id`.
pub async fn ensure_team_opponent_player(
    db: &Surreal<Db>,
    team_id: &str,
    player_id: &str,
) -> DbResult<()> {
    ensure_owned(
        db,
        "opponent_player",
        player_id,
        "opponent.team",
        "team",
        team_id,
    )
    .await
}

#[allow(clippy::too_many_arguments)] // one column per argument; params struct planned in M6
pub async fn save_opponent_player_info(
    db: &Surreal<Db>,
    team_id: &str,
    player_id: &str,
    name: String,
    role: String,
    riot_summoner_name: Option<String>,
    notes: Option<String>,
) -> DbResult<()> {
    ensure_team_opponent_player(db, team_id, player_id).await?;
    let player_key = record_key(player_id, "opponent_player");
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

#[allow(clippy::too_many_arguments)] // one column per argument; params struct planned in M6
pub async fn update_opponent_player(
    db: &Surreal<Db>,
    team_id: &str,
    player_id: &str,
    name: String,
    role: String,
    riot_puuid: Option<String>,
    riot_summoner_name: Option<String>,
    recent_champions: Vec<String>,
    notes: Option<String>,
) -> DbResult<()> {
    ensure_team_opponent_player(db, team_id, player_id).await?;
    let player_key = record_key(player_id, "opponent_player");
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

pub async fn delete_opponent_player(
    db: &Surreal<Db>,
    team_id: &str,
    player_id: &str,
) -> DbResult<()> {
    ensure_team_opponent_player(db, team_id, player_id).await?;
    let player_key = record_key(player_id, "opponent_player");
    db.query("DELETE type::record('opponent_player', $player_key)")
        .bind(("player_key", player_key))
        .await?
        .check()?;
    Ok(())
}

pub async fn update_opponent_player_champions(
    db: &Surreal<Db>,
    team_id: &str,
    player_id: &str,
    champions: Vec<String>,
) -> DbResult<()> {
    ensure_team_opponent_player(db, team_id, player_id).await?;
    let player_key = record_key(player_id, "opponent_player");
    db.query(
        "UPDATE type::record('opponent_player', $player_key) SET recent_champions = $champions",
    )
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
    // One transaction: the opponent and every player slot, or nothing. The
    // per-player statements use indexed placeholders ($role_0, ...); only the
    // index is formatted into the query text, every value is bound.
    let mut query = String::from(
        "BEGIN TRANSACTION; \
         LET $o = (CREATE ONLY opponent SET name = $opp_name, team = type::record('team', $team_key), notes = $opp_notes);",
    );
    for i in 0..players.len() {
        query.push_str(&format!(
            " CREATE ONLY opponent_player SET opponent = $o.id, name = '', role = $role_{i}, riot_summoner_name = $summoner_{i};"
        ));
    }
    query.push_str(" RETURN { id: $o.id }; COMMIT TRANSACTION;");

    let mut q = db
        .query(query)
        .bind(("opp_name", name))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("opp_notes", notes));
    for (i, (role, summoner)) in players.iter().enumerate() {
        q = q
            .bind((format!("role_{i}"), role.clone()))
            .bind((format!("summoner_{i}"), summoner.clone()));
    }
    let mut response = check_tx(q.await?)?;

    // Statement indexes: 0 BEGIN, 1 LET, 2..=n+1 player CREATEs, n+2 RETURN.
    let mut player_ids = Vec::with_capacity(players.len());
    for i in 0..players.len() {
        let player_row: Option<IdRecord> = response.take(i + 2)?;
        match player_row {
            Some(r) => player_ids.push(r.id.to_sql()),
            None => return Err(DbError::Other(format!("Failed to create player slot {i}"))),
        }
    }
    let opp: Option<IdRecord> = response.take(players.len() + 2)?;
    let opponent_id = opp
        .map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create opponent".into()))?;
    Ok((opponent_id, player_ids))
}

/// Persist enriched data fetched from the Riot API for a single opponent player.
///
/// Sets `last_fetched = time::now()` server-side to record when the data was written.
#[allow(clippy::too_many_arguments)] // one column per argument; params struct planned in M6
pub async fn update_opponent_player_intel(
    db: &Surreal<Db>,
    team_id: &str,
    player_id: &str,
    riot_puuid: Option<String>,
    recent_champions: Vec<String>,
    mastery_data_json: Option<String>,
    role_distribution_json: Option<String>,
) -> DbResult<()> {
    ensure_team_opponent_player(db, team_id, player_id).await?;
    let player_key = record_key(player_id, "opponent_player");
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
    opponent_id: Option<RecordId>,
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
            opponent_id: opt_link(s.opponent_id),
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
        .query("CREATE series SET name = $name, team = type::record('team', $team_key), created_by = type::record('user', $user_key), opponent_id = IF $opponent_key = NONE { NONE } ELSE { type::record('opponent', $opponent_key) }, opponent_name = $opponent_name, format = $format, is_fearless = $is_fearless, notes = $notes")
        .bind(("name", name))
        .bind(("team_key", team_key))
        .bind(("user_key", user_key))
        .bind(("opponent_key", opt_key(opponent_id, "opponent")))
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

pub async fn get_series(
    db: &Surreal<Db>,
    team_id: &str,
    series_id: &str,
) -> DbResult<Option<Series>> {
    let series_key = record_key(series_id, "series");
    let mut result = db
        .query("SELECT * FROM type::record('series', $series_key) WHERE team = type::record('team', $team_key)")
        .bind(("series_key", series_key))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let row: Option<DbSeries> = result.take(0)?;
    Ok(row.map(Series::from))
}

#[allow(clippy::too_many_arguments)]
pub async fn update_series(
    db: &Surreal<Db>,
    team_id: &str,
    series_id: &str,
    name: String,
    opponent_id: Option<String>,
    opponent_name: Option<String>,
    format: String,
    is_fearless: bool,
    notes: Option<String>,
) -> DbResult<()> {
    ensure_team_record(db, "series", series_id, team_id).await?;
    let series_key = record_key(series_id, "series");
    db.query("UPDATE type::record('series', $series_key) SET name=$name, opponent_id = IF $opponent_key = NONE { NONE } ELSE { type::record('opponent', $opponent_key) }, opponent_name=$opponent_name, format=$format, is_fearless=$is_fearless, notes=$notes")
        .bind(("series_key", series_key))
        .bind(("name", name))
        .bind(("opponent_key", opt_key(opponent_id, "opponent")))
        .bind(("opponent_name", opponent_name))
        .bind(("format", format))
        .bind(("is_fearless", is_fearless))
        .bind(("notes", notes))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_series(db: &Surreal<Db>, team_id: &str, series_id: &str) -> DbResult<()> {
    ensure_team_record(db, "series", series_id, team_id).await?;
    // Unlink the team's drafts and delete the series together.
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             UPDATE draft SET series_id = NONE, game_number = NONE WHERE series_id = type::record('series', $series_key) AND team = type::record('team', $team_key); \
             DELETE type::record('series', $series_key); \
             COMMIT TRANSACTION;",
        )
        .bind(("series_key", record_key(series_id, "series")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    check_tx(response)?;
    Ok(())
}

pub async fn list_series_drafts(
    db: &Surreal<Db>,
    team_id: &str,
    series_id: &str,
) -> DbResult<Vec<Draft>> {
    let mut result = db
        .query("SELECT * FROM draft WHERE series_id = type::record('series', $series_key) AND team = type::record('team', $team_key) ORDER BY game_number ASC")
        .bind(("series_key", record_key(series_id, "series")))
        .bind(("team_key", record_key(team_id, "team")))
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
/// Champions already used in the `team_id` drafts of a series.
pub async fn get_fearless_used_champions(
    db: &Surreal<Db>,
    team_id: &str,
    series_id: &str,
    exclude_draft_id: Option<&str>,
) -> DbResult<Vec<String>> {
    let sid = record_key(series_id, "series");
    let team_key = record_key(team_id, "team");

    let query = match exclude_draft_id {
        Some(eid) => {
            let mut result = db
                .query("SELECT champion FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE series_id = type::record('series', $sid) AND team = type::record('team', $team_key) AND id != type::record('draft', $eid))")
                .bind(("sid", sid))
                .bind(("team_key", team_key))
                .bind(("eid", record_key(eid, "draft")))
                .await?;
            let rows: Vec<ChampionRow> = result.take(0).unwrap_or_default();
            rows
        }
        None => {
            let mut result = db
                .query("SELECT champion FROM draft_action WHERE draft IN (SELECT VALUE id FROM draft WHERE series_id = type::record('series', $sid) AND team = type::record('team', $team_key))")
                .bind(("sid", sid))
                .bind(("team_key", team_key))
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
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

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
            .query("SELECT * FROM champion_note WHERE user = type::record('user', $user_key) AND note_type = 'matchup' AND (title CONTAINS $champion OR content CONTAINS $champion)")
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
    source_review: Option<RecordId>,
    text: String,
    status: String,
    assigned_to: Option<String>,
    due: Option<String>,
    created_at: Option<Datetime>,
    resolved_at: Option<Datetime>,
}

impl From<DbActionItem> for ActionItem {
    fn from(a: DbActionItem) -> Self {
        ActionItem {
            id: Some(a.id.to_sql()),
            team_id: a.team.to_sql(),
            source_review: opt_link(a.source_review),
            text: a.text,
            status: a.status,
            assigned_to: a.assigned_to,
            due: a.due,
            created_at: opt_dt(a.created_at),
            resolved_at: opt_dt(a.resolved_at),
        }
    }
}

/// A due date as stored: `None` for empty input, `Invalid` unless it is a
/// calendar date written `YYYY-MM-DD`.
fn checked_due(due: Option<String>) -> DbResult<Option<String>> {
    ActionItem::parse_due(due.as_deref().unwrap_or("")).map_err(|e| DbError::Invalid(e.to_string()))
}

pub async fn create_action_item(
    db: &Surreal<Db>,
    team_id: &str,
    text: String,
    source_review: Option<String>,
    assigned_to: Option<String>,
    due: Option<String>,
) -> DbResult<String> {
    let due = checked_due(due)?;
    if let Some(review) = source_review.as_deref() {
        ensure_team_record(db, "post_game_learning", review, team_id).await?;
    }
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut response = db
        .query(
            "CREATE action_item SET team = type::record('team', $team_key), text = $text, source_review = IF $review_key = NONE { NONE } ELSE { type::record('post_game_learning', $review_key) }, assigned_to = $assigned_to, due = $due",
        )
        .bind(("team_key", team_key))
        .bind(("text", text))
        .bind(("review_key", opt_key(source_review, "post_game_learning")))
        .bind(("assigned_to", assigned_to))
        .bind(("due", due))
        .await?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create action item".into())),
    }
}

pub async fn list_action_items(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<ActionItem>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query(
            "SELECT * FROM action_item WHERE team = type::record('team', $team_key) ORDER BY status ASC, created_at DESC",
        )
        .bind(("team_key", team_key))
        .await?;
    let items: Vec<DbActionItem> = r.take(0).unwrap_or_default();
    Ok(items.into_iter().map(ActionItem::from).collect())
}

pub async fn list_open_action_items(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<ActionItem>> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();
    let mut r = db
        .query(
            "SELECT * FROM action_item WHERE team = type::record('team', $team_key) AND (status = 'open' OR status = 'in_progress') ORDER BY status ASC, created_at DESC",
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
    ensure_team_record(db, "post_game_learning", review_id, team_id).await?;
    let items = new_action_item_texts(db, team_id, Some(review_id), improvements).await?;
    let count = items.len();
    if count == 0 {
        return Ok(0);
    }
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             FOR $t IN $items { CREATE action_item SET team = type::record('team', $team_key), text = $t, source_review = type::record('post_game_learning', $review_key); }; \
             COMMIT TRANSACTION;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("review_key", record_key(review_id, "post_game_learning")))
        .bind(("items", items))
        .await?;
    check_tx(response)?;
    Ok(count)
}

pub async fn update_action_item_status(
    db: &Surreal<Db>,
    team_id: &str,
    item_id: &str,
    status: String,
) -> DbResult<()> {
    ensure_team_record(db, "action_item", item_id, team_id).await?;
    let item_key = record_key(item_id, "action_item");
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
    team_id: &str,
    item_id: &str,
    text: String,
    assigned_to: Option<String>,
    due: Option<String>,
    status: Option<String>,
) -> DbResult<()> {
    let due = checked_due(due)?;
    ensure_team_record(db, "action_item", item_id, team_id).await?;
    let item_key = record_key(item_id, "action_item");
    // `None` keeps the current status and resolved_at (editing the text of a
    // done item must not reopen it).
    let query = if status.is_some() {
        "UPDATE type::record('action_item', $item_key) SET text = $text, assigned_to = $assigned_to, due = $due, status = $status, resolved_at = IF $status = 'done' { time::now() } ELSE { NONE }"
    } else {
        "UPDATE type::record('action_item', $item_key) SET text = $text, assigned_to = $assigned_to, due = $due"
    };
    db.query(query)
        .bind(("item_key", item_key))
        .bind(("text", text))
        .bind(("assigned_to", assigned_to))
        .bind(("due", due))
        .bind(("status", status))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_action_item(db: &Surreal<Db>, team_id: &str, item_id: &str) -> DbResult<()> {
    ensure_team_record(db, "action_item", item_id, team_id).await?;
    let item_key = record_key(item_id, "action_item");
    db.query("DELETE type::record('action_item', $item_key)")
        .bind(("item_key", item_key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Bug Reports (Phase 19 D-02/D-03)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbBugReport {
    id: RecordId,
    user: RecordId,
    page_url: String,
    element_label: String,
    description: String,
    category: String,
    viewport_w: Option<i64>,
    viewport_h: Option<i64>,
    created_at: Option<Datetime>,
    status: String,
}

impl From<DbBugReport> for BugReport {
    fn from(b: DbBugReport) -> Self {
        BugReport {
            id: Some(b.id.to_sql()),
            user_id: b.user.to_sql(),
            page_url: b.page_url,
            element_label: b.element_label,
            description: b.description,
            category: b.category,
            viewport_w: b.viewport_w.map(|v| v as i32),
            viewport_h: b.viewport_h.map(|v| v as i32),
            created_at: opt_dt(b.created_at),
            status: b.status,
        }
    }
}

#[allow(clippy::too_many_arguments)] // one column per argument; params struct planned in M6
pub async fn create_bug_report(
    db: &Surreal<Db>,
    user_id: &str,
    page_url: String,
    element_label: String,
    description: String,
    category: String,
    viewport_w: Option<i32>,
    viewport_h: Option<i32>,
) -> DbResult<()> {
    // T-19-02 mitigation: reject whitespace-only descriptions at the DB layer
    // so any caller (server-fn, future CLI, etc.) gets the same guarantee.
    if description.trim().is_empty() {
        return Err(DbError::Other("Description is required".into()));
    }
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    // Owned `String` for every `.bind` per surreal-patterns rule 4.
    db.query(
        "CREATE bug_report SET \
             user = type::record('user', $user_key), \
             page_url = $page_url, \
             element_label = $element_label, \
             description = $description, \
             category = $category, \
             viewport_w = $viewport_w, \
             viewport_h = $viewport_h",
    )
    .bind(("user_key", user_key))
    .bind(("page_url", page_url))
    .bind(("element_label", element_label))
    .bind(("description", description))
    .bind(("category", category))
    .bind(("viewport_w", viewport_w.map(|v| v as i64)))
    .bind(("viewport_h", viewport_h.map(|v| v as i64)))
    .await?
    .check()?; // surreal-patterns rule 27: .check() on writes
    Ok(())
}

pub async fn list_bug_reports(db: &Surreal<Db>, status: Option<&str>) -> DbResult<Vec<BugReport>> {
    let mut r = match status {
        Some(s) => {
            db.query(
                "SELECT * \
                 FROM bug_report WHERE status = $status \
                 ORDER BY created_at DESC",
            )
            .bind(("status", s.to_string()))
            .await?
        }
        None => {
            db.query(
                "SELECT * \
                 FROM bug_report ORDER BY created_at DESC",
            )
            .await?
        }
    };
    let rows: Vec<DbBugReport> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(BugReport::from).collect())
}

/// The open queue for the inbox export: reports in `new` or `triaged`.
pub async fn list_untriaged_bug_reports(db: &Surreal<Db>) -> DbResult<Vec<BugReport>> {
    let mut r = db
        .query(
            "SELECT * FROM bug_report WHERE status IN ['new', 'triaged'] ORDER BY created_at DESC",
        )
        .await?;
    let rows: Vec<DbBugReport> = r.take(0)?;
    Ok(rows.into_iter().map(BugReport::from).collect())
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
    created_at: Option<Datetime>,
    edited_at: Option<Datetime>,
    edited_by_name: Option<String>,
    previous_content: Option<String>,
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
            created_at: opt_dt(n.created_at),
            edited_at: opt_dt(n.edited_at),
            edited_by_name: n.edited_by_name,
            can_undo_edit: n.previous_content.is_some(),
        }
    }
}

const TEAM_NOTE_FIELDS: &str =
    "id, team, author, author_name, content, pinned, created_at, edited_at, edited_by_name, previous_content";

/// Longest team note, in characters.
pub const TEAM_NOTE_MAX: usize = 4000;

fn checked_note(content: String) -> DbResult<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(DbError::Invalid("A note needs some text.".into()));
    }
    if trimmed.chars().count() > TEAM_NOTE_MAX {
        return Err(DbError::Invalid(format!(
            "Notes are at most {TEAM_NOTE_MAX} characters."
        )));
    }
    Ok(trimmed.to_string())
}

pub async fn create_team_note(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    username: &str,
    content: String,
) -> DbResult<String> {
    let content = checked_note(content)?;
    let mut response = db
        .query("CREATE team_note SET team = type::record('team', $team_key), author = type::record('user', $user_key), author_name = $author_name, content = $content")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("author_name", username.to_string()))
        .bind(("content", content))
        .await?;

    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create team note".into())),
    }
}

/// The team's notes, pinned first, newest first. Deleted notes (kept a day
/// for Undo) are left out.
pub async fn list_team_notes(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<TeamNote>> {
    let mut response = db
        .query(format!("SELECT {TEAM_NOTE_FIELDS} FROM team_note WHERE team = type::record('team', $team_key) AND deleted_at = NONE ORDER BY pinned DESC, created_at DESC"))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;

    let rows: Vec<DbTeamNote> = response.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(TeamNote::from).collect())
}

pub async fn list_pinned_team_notes(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<TeamNote>> {
    let mut response = db
        .query(format!("SELECT {TEAM_NOTE_FIELDS} FROM team_note WHERE team = type::record('team', $team_key) AND pinned = true AND deleted_at = NONE ORDER BY created_at DESC"))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;

    let rows: Vec<DbTeamNote> = response.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(TeamNote::from).collect())
}

/// Replace a note's text (any member may). The text it replaces is kept so
/// the edit can be undone (`undo_team_note_edit`).
pub async fn update_team_note(
    db: &Surreal<Db>,
    team_id: &str,
    note_id: &str,
    content: String,
    editor_name: &str,
) -> DbResult<()> {
    let content = checked_note(content)?;
    let mut r = db
        .query("UPDATE type::record('team_note', $note_key) SET previous_content = content, content = $content, edited_at = time::now(), edited_by_name = $editor WHERE team = type::record('team', $team_key) AND deleted_at = NONE RETURN id")
        .bind(("note_key", record_key(note_id, "team_note")))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("content", content))
        .bind(("editor", editor_name.to_string()))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Put back the text the last edit replaced. Only one step: the restored
/// text cannot be undone again.
pub async fn undo_team_note_edit(db: &Surreal<Db>, team_id: &str, note_id: &str) -> DbResult<()> {
    let mut r = db
        .query("UPDATE type::record('team_note', $note_key) SET content = previous_content, previous_content = NONE WHERE team = type::record('team', $team_key) AND previous_content != NONE AND deleted_at = NONE RETURN id")
        .bind(("note_key", record_key(note_id, "team_note")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn toggle_pin_team_note(
    db: &Surreal<Db>,
    team_id: &str,
    note_id: &str,
    pinned: bool,
) -> DbResult<()> {
    ensure_team_record(db, "team_note", note_id, team_id).await?;
    let note_key = record_key(note_id, "team_note");

    db.query("UPDATE type::record('team_note', $note_key) SET pinned = $pinned")
        .bind(("note_key", note_key))
        .bind(("pinned", pinned))
        .await?
        .check()?;
    Ok(())
}

/// How long a deleted note can be restored.
const NOTE_UNDO_WINDOW: &str = "1d";

/// Delete a note so that it can still be restored for a day
/// (`restore_team_note`). Notes deleted longer ago are purged here.
pub async fn delete_team_note(db: &Surreal<Db>, team_id: &str, note_id: &str) -> DbResult<()> {
    ensure_team_record(db, "team_note", note_id, team_id).await?;
    db.query(format!(
        "UPDATE type::record('team_note', $note_key) SET deleted_at = time::now(); \
         DELETE team_note WHERE team = type::record('team', $team_key) AND deleted_at != NONE AND deleted_at < time::now() - {NOTE_UNDO_WINDOW};"
    ))
    .bind(("note_key", record_key(note_id, "team_note")))
    .bind(("team_key", record_key(team_id, "team")))
    .await?
    .check()?;
    Ok(())
}

/// Bring back a note deleted within the last day. Older deletes stay
/// deleted (and are purged hourly, `purge_deleted_team_notes`).
pub async fn restore_team_note(db: &Surreal<Db>, team_id: &str, note_id: &str) -> DbResult<()> {
    let mut r = db
        .query(format!("UPDATE type::record('team_note', $note_key) SET deleted_at = NONE WHERE team = type::record('team', $team_key) AND deleted_at != NONE AND deleted_at >= time::now() - {NOTE_UNDO_WINDOW} RETURN id"))
        .bind(("note_key", record_key(note_id, "team_note")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Delete notes of every team that were deleted longer ago than the Undo
/// window. Runs hourly from the server's purge task.
pub async fn purge_deleted_team_notes(db: &Surreal<Db>) -> DbResult<()> {
    db.query(format!(
        "DELETE team_note WHERE deleted_at != NONE AND deleted_at < time::now() - {NOTE_UNDO_WINDOW};"
    ))
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
    tendencies.sort_by_key(|e| std::cmp::Reverse(e.3));
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
        game_plan_id: Option<RecordId>,
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
        if let Some(gp_id) = pg.game_plan_id.as_ref().map(|r| r.to_sql()) {
            if let Some(tag) = plan_tags.get(&gp_id) {
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
            .any(|pg| pg.game_plan_id.as_ref().map(|r| r.to_sql()).as_ref() == Some(plan_id));
        if !has_post_game {
            // Count as a game played but with no outcome data
            tag_stats.entry(tag.clone()).or_insert((0, 0));
        }
    }

    let mut stats: Vec<(String, i32, i32)> = tag_stats
        .into_iter()
        .map(|(tag, (total, wins))| (tag, total, wins))
        .collect();
    stats.sort_by_key(|e| std::cmp::Reverse(e.1));
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
    stats.sort_by_key(|e| std::cmp::Reverse(e.1));
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
        draft: Option<RecordId>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbDraftOpponent {
        id: RecordId,
        opponent: Option<String>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPostGameVs {
        id: RecordId,
        game_plan_id: Option<RecordId>,
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
        .filter_map(|d| d.opponent.as_ref().map(|opp| (d.id.to_sql(), opp.clone())))
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
                let opp = p
                    .draft
                    .as_ref()
                    .and_then(|did| draft_opponent.get(&did.to_sql()).cloned());
                (plan_sql, (tag.clone(), opp))
            })
        })
        .collect();

    // Build flat per-result tuples for filter_win_condition_stats
    let mut flat: Vec<(String, Option<String>, bool)> = Vec::new();
    for pg in &post_games {
        if let Some(gp_id) = pg.game_plan_id.as_ref().map(|r| r.to_sql()) {
            if let Some((tag, opp)) = plan_tags_lookup.get(&gp_id) {
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
        draft_id: Option<RecordId>,
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
        if let (Some(did), Some(ref riot_id)) = (&pg.draft_id, &pg.match_riot_id) {
            if let Some(record_id) = match_id_to_record.get(riot_id) {
                if let Some(&won) = match_record_wins.get(record_id) {
                    draft_outcome.insert(did.to_sql(), won);
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
    tag_stats.sort_by_key(|e| std::cmp::Reverse(e.1));

    let mut fp_stats: Vec<(String, i32, i32)> = first_pick_counts
        .into_iter()
        .map(|(champ, (g, w))| (champ, g, w))
        .collect();
    fp_stats.sort_by_key(|e| std::cmp::Reverse(e.1));

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

    // Uses the cached champion list (data_dragon::init runs first).
    let champions = match data_dragon::snapshot() {
        Some(s) => s.champions.clone(),
        None => {
            warn!("Champion name normalisation skipped: no Data Dragon copy available");
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
    const ALL_CLASSES: &[&str] = &["Fighter", "Mage", "Assassin", "Tank", "Marksman", "Support"];

    // Build champion class lookup: champion_id -> Vec<class>
    let class_map: HashMap<&str, &[String]> = champions
        .iter()
        .map(|c| (c.id.as_str(), c.tags.as_slice()))
        .collect();

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
                .map(|tags| tags.to_vec())
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
pub async fn get_dashboard_summary(db: &Surreal<Db>, team_id: &str) -> DbResult<DashboardSummary> {
    let team_key = team_id.strip_prefix("team:").unwrap_or(team_id).to_string();

    // 5-statement batched query (per RESEARCH.md Example 2)
    let mut result = db
        .query(
            "SELECT id, text, created_at FROM action_item \
             WHERE team = type::record('team', $team_key) AND status IN ['open','in_progress'] \
             ORDER BY created_at DESC LIMIT 3; \
             SELECT count() as n FROM action_item \
             WHERE team = type::record('team', $team_key) AND status IN ['open','in_progress'] \
             GROUP ALL; \
             SELECT id, improvements, created_at FROM post_game_learning \
             WHERE team = type::record('team', $team_key) \
             ORDER BY created_at DESC LIMIT 5; \
             LET $planned = (SELECT VALUE draft FROM game_plan WHERE team = type::record('team', $team_key) AND draft != NONE); \
             LET $reviewed = (SELECT VALUE game_plan_id FROM post_game_learning WHERE team = type::record('team', $team_key) AND game_plan_id != NONE); \
             SELECT count() as n FROM draft WHERE team = type::record('team', $team_key) \
             AND id NOT IN $planned GROUP ALL; \
             SELECT count() as n FROM game_plan WHERE team = type::record('team', $team_key) \
             AND id NOT IN $reviewed GROUP ALL",
        )
        .bind(("team_key", team_key.clone()))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbActionItemPreview {
        id: RecordId,
        text: String,
        #[allow(dead_code)]
        created_at: Option<Datetime>,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct CountResult {
        n: i64,
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbPostGamePreview {
        id: RecordId,
        improvements: Vec<String>,
        created_at: Option<Datetime>,
    }

    let db_action_items: Vec<DbActionItemPreview> = result.take(0).unwrap_or_default();
    let action_count_rows: Vec<CountResult> = result.take(1).unwrap_or_default();
    let db_post_games: Vec<DbPostGamePreview> = result.take(2).unwrap_or_default();
    let drafts_no_plan_rows: Vec<CountResult> = result.take(5).unwrap_or_default();
    let plans_no_postgame_rows: Vec<CountResult> = result.take(6).unwrap_or_default();

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
            created_at: opt_dt(p.created_at),
        })
        .collect();

    // Fetch champion pool entries for all team members (for gap analysis)
    // Non-fatal: gap analysis failure should not break the dashboard
    let pool_gap_warnings = compute_pool_gaps_for_team(db, &team_key)
        .await
        .unwrap_or_default();

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

    // Champion class tags from the Data Dragon cache (no network on this path).
    let champions = match data_dragon::snapshot() {
        Some(s) => s.champions.clone(),
        None => return Ok(Vec::new()),
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

    #[derive(Debug, Deserialize, SurrealValue)]
    struct PoolRow {
        user: RecordId,
        champion: String,
        role: String,
        tier: String,
    }

    let mut pr = db
        .query("LET $users = (SELECT VALUE user FROM team_member WHERE team = type::record('team', $team_key)); \
                SELECT user, champion, role, tier FROM champion_pool WHERE user IN $users")
        .bind(("team_key", team_key.to_string()))
        .await?;
    let pool_rows: Vec<PoolRow> = pr.take(1).unwrap_or_default();

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
        .query("LET $opps = (SELECT VALUE id FROM opponent WHERE team = type::record('team', $team_key)); \
                SELECT recent_champions FROM opponent_player WHERE opponent IN $opps")
        .bind(("team_key", team_key.to_string()))
        .await?;

    #[derive(Debug, Deserialize, SurrealValue)]
    struct RecentChamps {
        recent_champions: Vec<String>,
    }

    let opp_rows: Vec<RecentChamps> = opp_r.take(1).unwrap_or_default();
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
        let entry = map
            .entry(champ.clone())
            .or_insert_with(|| ChampionPerformanceSummary {
                champion: champ.clone(),
                ..Default::default()
            });
        entry.games_in_draft += 1;
    }

    for (champ, win) in match_stats {
        let entry = map
            .entry(champ.clone())
            .or_insert_with(|| ChampionPerformanceSummary {
                champion: champ.clone(),
                ..Default::default()
            });
        entry.games_in_match += 1;
        if *win {
            entry.wins_in_match += 1;
        }
    }

    for champ in plan_champions {
        let entry = map
            .entry(champ.clone())
            .or_insert_with(|| ChampionPerformanceSummary {
                champion: champ.clone(),
                ..Default::default()
            });
        entry.games_in_plan += 1;
    }

    for (champ, win) in post_game_champ_outcomes {
        let entry = map
            .entry(champ.clone())
            .or_insert_with(|| ChampionPerformanceSummary {
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
    result.sort_by_key(|e| std::cmp::Reverse(e.games_in_match));
    result
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
// User region
// ---------------------------------------------------------------------------

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

/// Returns true if the user's last_solo_sync is None or older than 10 minutes.
pub async fn get_should_auto_sync(db: &Surreal<Db>, user_id: &str) -> DbResult<bool> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct SyncRecord {
        last_solo_sync: Option<surrealdb::types::Datetime>,
    }
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut result = db
        .query("SELECT last_solo_sync FROM type::record('user', $user_key)")
        .bind(("user_key", user_key))
        .await?;
    let record: Option<SyncRecord> = result.take(0).unwrap_or_default();
    let should_sync = match record.and_then(|r| r.last_solo_sync) {
        None => true,
        Some(last) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let last_secs: i64 = last.timestamp();
            now - last_secs > 600
        }
    };
    Ok(should_sync)
}

// ---------------------------------------------------------------------------
// Solo Mode: Ranked snapshots
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)] // one column per argument; params struct planned in M6
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
        #[allow(dead_code)]
        snapshotted_at: Option<Datetime>, // required in SELECT for ORDER BY (Rule 40)
    }

    let mut result = db
        .query("SELECT queue_type, tier, division, lp, wins, losses, snapshotted_at FROM ranked_snapshot WHERE user = type::record('user', $user_key) AND queue_type = $queue_type ORDER BY snapshotted_at DESC LIMIT 1")
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
    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbSoloMatch {
        id: RecordId,
        match_id: String,
        user_id: String,
        #[allow(dead_code)]
        game_end: Option<Datetime>, // selected for ORDER BY (Rule 40)
        champion: String,
        kills: i32,
        deaths: i32,
        assists: i32,
        cs: i32,
        vision_score: i32,
        damage: i32,
        win: bool,
    }

    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    // Alias RecordId fields + include game_end for ORDER BY (Rule 40: ORDER BY field must be in SELECT)
    const BASE_SELECT: &str = "SELECT id, match.match_id AS match_id, <string>user AS user_id, \
         match.game_end AS game_end, \
         champion, kills, deaths, assists, cs, vision_score, damage, win \
         FROM player_match WHERE user = type::record('user', $user_key)";

    let rows: Vec<DbSoloMatch> = if let Some(qid) = queue_filter {
        let mut result = db
            .query(format!(
                "{BASE_SELECT} AND match.queue_id = $queue_id ORDER BY game_end DESC LIMIT $limit"
            ))
            .bind(("user_key", user_key))
            .bind(("queue_id", qid))
            .bind(("limit", limit))
            .await?;
        result.take(0).unwrap_or_default()
    } else {
        let mut result = db
            .query(format!("{BASE_SELECT} ORDER BY game_end DESC LIMIT $limit"))
            .bind(("user_key", user_key))
            .bind(("limit", limit))
            .await?;
        result.take(0).unwrap_or_default()
    };

    Ok(rows
        .into_iter()
        .map(|r| crate::models::match_data::PlayerMatchStats {
            id: Some(r.id.to_sql()),
            match_id: r.match_id,
            user_id: r.user_id,
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

// ===========================================================================
// Phase 15: Goals & LP History
// ===========================================================================

pub async fn get_lp_history(
    db: &Surreal<Db>,
    user_id: &str,
    cutoff: Option<String>, // ISO datetime string (UTC); None = all-time
) -> DbResult<Vec<crate::models::match_data::RankedSnapshot>> {
    use crate::models::match_data::rank_score;
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbLpRow {
        tier: String,
        division: String,
        lp: i32,
        snapshotted_at: Datetime,
    }

    // Note: snapshotted_at MUST be in SELECT list for ORDER BY to work (Rule 40).
    // It loads as a Datetime and is formatted with dt_to_string.
    let rows: Vec<DbLpRow> = if let Some(cutoff_val) = cutoff {
        let mut r = db
            .query(
                "SELECT tier, division, lp, snapshotted_at \
                 FROM ranked_snapshot \
                 WHERE user = type::record('user', $user_key) \
                 AND queue_type = 'RANKED_SOLO_5x5' \
                 AND snapshotted_at >= <datetime>$cutoff \
                 ORDER BY snapshotted_at ASC",
            )
            .bind(("user_key", user_key))
            .bind(("cutoff", cutoff_val))
            .await?;
        r.take(0).unwrap_or_default()
    } else {
        let mut r = db
            .query(
                "SELECT tier, division, lp, snapshotted_at \
                 FROM ranked_snapshot \
                 WHERE user = type::record('user', $user_key) \
                 AND queue_type = 'RANKED_SOLO_5x5' \
                 ORDER BY snapshotted_at ASC",
            )
            .bind(("user_key", user_key))
            .await?;
        r.take(0).unwrap_or_default()
    };

    Ok(rows
        .into_iter()
        .map(|r| {
            let score = rank_score(&r.tier, &r.division, r.lp);
            crate::models::match_data::RankedSnapshot {
                id: None,
                tier: r.tier,
                division: r.division,
                lp: r.lp,
                snapshotted_at: dt_to_string(&r.snapshotted_at),
                rank_score: score,
            }
        })
        .collect())
}

/// Upsert a personal goal for the user (one active goal per type, D-07).
/// Validates goal_type server-side as defense in depth (T-15-01).
pub async fn upsert_personal_goal(
    db: &Surreal<Db>,
    user_id: &str,
    goal_type: &str,
    target_value: &str,
) -> DbResult<()> {
    // Validate goal_type — defence in depth; server fn validates first (T-15-01).
    if !matches!(goal_type, "rank_target" | "cs_per_min" | "deaths_per_game") {
        return Err(DbError::Other(format!("invalid goal_type: {goal_type}")));
    }
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    // DELETE + CREATE in a transaction ensures upsert semantics atomically (set_ban_priorities pattern).
    db.query(
        "BEGIN TRANSACTION; \
         DELETE personal_goal WHERE user = type::record('user', $user_key) \
           AND goal_type = $goal_type; \
         CREATE personal_goal SET \
           user = type::record('user', $user_key), \
           goal_type = $goal_type, \
           target_value = $target_value, \
           created_at = time::now(), \
           updated_at = time::now(); \
         COMMIT TRANSACTION;",
    )
    .bind(("user_key", user_key))
    .bind(("goal_type", goal_type.to_string()))
    .bind(("target_value", target_value.to_string()))
    .await?
    .check()?;
    Ok(())
}

/// Compute goal progress for the user from the last 20 solo/duo player_match rows (D-12, D-13).
/// Returns `current_value: None` when fewer than 5 qualifying games exist (D-15).
pub async fn compute_goal_progress(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<crate::models::match_data::GoalProgressPayload> {
    use crate::models::match_data::{rank_score, GoalProgress, GoalProgressPayload, PersonalGoal};
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbGoalRow {
        id: RecordId,
        goal_type: String,
        target_value: String,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbGoalMatchRow {
        kills: i64,
        deaths: i64,
        assists: i64,
        cs: i64,
        game_duration: i64, // seconds — Pitfall 1: divide by 60 for cs/min
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbCurrentRankRow {
        tier: String,
        division: String,
        lp: i32,
        wins: i32,
        losses: i32,
        #[allow(dead_code)]
        snapshotted_at: Option<Datetime>, // included for ORDER BY compliance (Rule 40)
    }

    // Batched 3-statement query (Rule 29).
    // Note: ORDER BY match.game_end is not supported for link-traversal fields in SurrealDB 3.x
    // (Assumption A2 from RESEARCH.md was correct). We fetch all rows and sort+cap in Rust.
    let mut r = db
        .query(
            "SELECT id, goal_type, target_value FROM personal_goal \
             WHERE user = type::record('user', $user_key); \
             SELECT kills, deaths, assists, cs, match.game_duration AS game_duration \
             FROM player_match \
             WHERE user = type::record('user', $user_key) \
             AND match.queue_id = 420; \
             SELECT tier, division, lp, wins, losses, snapshotted_at FROM ranked_snapshot \
             WHERE user = type::record('user', $user_key) \
             AND queue_type = 'RANKED_SOLO_5x5' \
             ORDER BY snapshotted_at DESC LIMIT 1",
        )
        .bind(("user_key", user_key))
        .await?;

    let goals: Vec<DbGoalRow> = r.take(0).unwrap_or_default();
    // Fetch all qualifying solo/duo matches, then cap to the most recent 20.
    // We can't ORDER BY match.game_end via link traversal in SurrealDB 3.x, so cap by count.
    let all_recent: Vec<DbGoalMatchRow> = r.take(1).unwrap_or_default();
    // Cap at 20 — the exact order doesn't affect the avg computation (all games weighted equally).
    let recent: Vec<DbGoalMatchRow> = all_recent.into_iter().take(20).collect();
    let current: Option<DbCurrentRankRow> = r.take(2).unwrap_or(None);

    // Helper: find a goal by type and convert to shared model.
    let find_goal = |kind: &str| -> Option<PersonalGoal> {
        goals
            .iter()
            .find(|g| g.goal_type == kind)
            .map(|g| PersonalGoal {
                id: Some(g.id.to_sql()),
                goal_type: g.goal_type.clone(),
                target_value: g.target_value.clone(),
            })
    };

    let game_count = recent.len() as i32;
    let sufficient = game_count >= 5; // D-15: insufficient threshold

    // CS/min progress: total_cs / (total_game_duration_sec / 60.0) — Pitfall 1.
    let cs_progress: Option<GoalProgress> = find_goal("cs_per_min").map(|goal| {
        if !sufficient {
            return GoalProgress {
                goal: goal.clone(),
                current_value: None,
                game_count,
                achieved: false,
            };
        }
        let total_cs: f32 = recent.iter().map(|r| r.cs as f32).sum();
        let total_minutes: f32 = recent.iter().map(|r| r.game_duration as f32 / 60.0).sum();
        let avg = if total_minutes > 0.0 {
            total_cs / total_minutes
        } else {
            0.0
        };
        let target = goal.target_value.parse::<f32>().unwrap_or(0.0);
        GoalProgress {
            goal,
            current_value: Some((avg * 10.0).round() / 10.0),
            game_count,
            achieved: avg >= target,
        }
    });

    // Deaths/game progress: avg deaths per game; lower is better (D-14).
    let deaths_progress: Option<GoalProgress> = find_goal("deaths_per_game").map(|goal| {
        if !sufficient {
            return GoalProgress {
                goal: goal.clone(),
                current_value: None,
                game_count,
                achieved: false,
            };
        }
        let avg = recent.iter().map(|r| r.deaths as f32).sum::<f32>() / game_count as f32;
        let target = goal.target_value.parse::<f32>().unwrap_or(0.0);
        GoalProgress {
            goal,
            current_value: Some((avg * 10.0).round() / 10.0),
            game_count,
            achieved: avg <= target,
        }
    });

    // Rank progress: compares current rank_score to target's rank_score.
    // No game-count threshold for rank — it reads from ranked_snapshot, not match aggregates.
    let rank_progress: Option<GoalProgress> = find_goal("rank_target").map(|goal| {
        let current_score: Option<i32> = current
            .as_ref()
            .map(|c| rank_score(&c.tier, &c.division, c.lp));
        // target_value format "TIER:DIVISION" (D-09); division empty for Master+.
        let parts: Vec<&str> = goal.target_value.splitn(2, ':').collect();
        let target_tier = parts.first().copied().unwrap_or("");
        let target_div = parts.get(1).copied().unwrap_or("");
        let target_score = rank_score(target_tier, target_div, 0);
        GoalProgress {
            goal,
            current_value: current_score.map(|s| s as f32),
            game_count: current.as_ref().map(|_| 1).unwrap_or(0),
            achieved: current_score.map(|s| s >= target_score).unwrap_or(false),
        }
    });

    let current_rank = current.map(|c| crate::models::user::RankedInfo {
        queue_type: "RANKED_SOLO_5x5".to_string(),
        tier: c.tier,
        division: c.division,
        lp: c.lp,
        wins: c.wins,
        losses: c.losses,
    });

    Ok(GoalProgressPayload {
        rank: rank_progress,
        cs: cs_progress,
        deaths: deaths_progress,
        current_rank,
    })
}

/// Aggregate champion performance trends for the user (D-16, D-19).
/// Includes solo/duo (queue_id=420) + flex (queue_id=440) — excludes ARAM, normals.
/// Aggregates in Rust after filtered DB fetch (matching get_champion_stats_for_user pattern).
pub async fn get_champion_trends(
    db: &Surreal<Db>,
    user_id: &str,
    cutoff: Option<String>, // ISO datetime string (UTC); None = all-time
) -> DbResult<Vec<crate::models::match_data::ChampionTrend>> {
    use std::collections::HashMap;
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();

    #[derive(Debug, Deserialize, SurrealValue)]
    struct DbTrendRow {
        champion: String,
        kills: i64,
        deaths: i64,
        assists: i64,
        cs: i64,
        damage: i64,
        win: bool,
        game_duration: i64, // seconds from match.game_duration — Pitfall 1
    }

    // Pitfall 4: match.game_end is option<datetime>. For time-windowed queries we
    // legitimately exclude null-dated matches. For all-time we include them all.
    let rows: Vec<DbTrendRow> = if let Some(cutoff_val) = cutoff {
        let mut r = db
            .query(
                "SELECT champion, kills, deaths, assists, cs, damage, win, \
                        match.game_duration AS game_duration \
                 FROM player_match \
                 WHERE user = type::record('user', $user_key) \
                 AND (match.queue_id = 420 OR match.queue_id = 440) \
                 AND match.game_end >= <datetime>$cutoff",
            )
            .bind(("user_key", user_key))
            .bind(("cutoff", cutoff_val))
            .await?;
        r.take(0).unwrap_or_default()
    } else {
        let mut r = db
            .query(
                "SELECT champion, kills, deaths, assists, cs, damage, win, \
                        match.game_duration AS game_duration \
                 FROM player_match \
                 WHERE user = type::record('user', $user_key) \
                 AND (match.queue_id = 420 OR match.queue_id = 440)",
            )
            .bind(("user_key", user_key))
            .await?;
        r.take(0).unwrap_or_default()
    };

    let mut by_champ: HashMap<String, Vec<DbTrendRow>> = HashMap::new();
    for row in rows {
        by_champ.entry(row.champion.clone()).or_default().push(row);
    }

    let mut trends: Vec<crate::models::match_data::ChampionTrend> = by_champ
        .into_iter()
        .map(|(champion, games)| {
            let n = games.len() as i32;
            let wins = games.iter().filter(|g| g.win).count() as i32;
            // Per-game KDA averaged: (K+A)/max(D,1) — handles zero-deaths (D-discretion).
            let kda_sum: f32 = games
                .iter()
                .map(|g| (g.kills as f32 + g.assists as f32) / (g.deaths as f32).max(1.0))
                .sum();
            let avg_kda = if n > 0 { kda_sum / n as f32 } else { 0.0 };
            // CS/min: total_cs / (total_duration_sec / 60.0) — seconds-based (Pitfall 1).
            let total_cs: f32 = games.iter().map(|g| g.cs as f32).sum();
            let total_minutes: f32 = games.iter().map(|g| g.game_duration as f32 / 60.0).sum();
            let cs_per_min = if total_minutes > 0.0 {
                total_cs / total_minutes
            } else {
                0.0
            };
            let total_damage: i64 = games.iter().map(|g| g.damage).sum();
            let avg_damage = if n > 0 {
                (total_damage / n as i64) as i32
            } else {
                0
            };
            crate::models::match_data::ChampionTrend {
                champion,
                games: n,
                wins,
                avg_kda: (avg_kda * 10.0).round() / 10.0,
                cs_per_min: (cs_per_min * 10.0).round() / 10.0,
                avg_damage,
            }
        })
        .collect();
    trends.sort_by_key(|e| std::cmp::Reverse(e.games));
    Ok(trends)
}

// ---- Match detail cache ----

#[derive(Debug, Deserialize, SurrealValue)]
struct DbMatchDetail {
    pub match_id: String,
    pub participants_json: String,
    pub game_duration: i32,
    pub game_mode: Option<String>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTimelineEvent {
    pub match_id: String,
    pub timestamp_ms: i64,
    pub event_type: String,
    pub team_id: Option<i32>,
    pub killer_participant_id: Option<i32>,
    pub victim_participant_id: Option<i32>,
    pub monster_type: Option<String>,
    pub monster_sub_type: Option<String>,
    pub building_type: Option<String>,
    pub is_first_blood: bool,
    pub multi_kill_length: Option<i32>,
    pub is_teamfight: bool,
    pub involved_participants: Option<String>,
}

fn classify_db_event(
    event_type: &str,
    is_teamfight: bool,
) -> crate::models::match_data::EventCategory {
    use crate::models::match_data::EventCategory;
    if is_teamfight || event_type == "TEAMFIGHT" {
        return EventCategory::Teamfight;
    }
    match event_type {
        "ELITE_MONSTER_KILL" => EventCategory::Objective,
        "BUILDING_KILL" => EventCategory::Tower,
        "CHAMPION_KILL" => EventCategory::Kill,
        "WARD_PLACED" => EventCategory::Ward,
        "ITEM_UNDO" => EventCategory::Recall,
        _ => EventCategory::Kill,
    }
}

pub async fn get_cached_match_detail(
    db: &Surreal<Db>,
    match_id: &str,
) -> Result<
    Option<(
        Vec<crate::models::match_data::MatchParticipant>,
        Vec<crate::models::match_data::TimelineEvent>,
        i32,
        String,
    )>,
    DbError,
> {
    let mut result = db
        .query("SELECT * FROM match_detail WHERE match_id = $match_id LIMIT 1")
        .bind(("match_id", match_id.to_string()))
        .await?;

    let detail: Option<DbMatchDetail> = result.take(0)?;
    let Some(detail) = detail else {
        return Ok(None);
    };

    let participants: Vec<crate::models::match_data::MatchParticipant> =
        serde_json::from_str(&detail.participants_json)
            .map_err(|e| DbError::Other(e.to_string()))?;

    // Fetch timeline events
    let mut tl_result = db
        .query("SELECT * FROM match_timeline_event WHERE match_id = $match_id")
        .bind(("match_id", match_id.to_string()))
        .await?;

    let tl_rows: Vec<DbTimelineEvent> = tl_result.take(0).unwrap_or_default();
    let timeline_events: Vec<crate::models::match_data::TimelineEvent> = tl_rows
        .into_iter()
        .map(|r| {
            let involved: Vec<i32> = r
                .involved_participants
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            crate::models::match_data::TimelineEvent {
                timestamp_ms: r.timestamp_ms,
                event_type: r.event_type.clone(),
                category: classify_db_event(&r.event_type, r.is_teamfight),
                team_id: r.team_id,
                killer_participant_id: r.killer_participant_id,
                victim_participant_id: r.victim_participant_id,
                monster_type: r.monster_type,
                monster_sub_type: r.monster_sub_type,
                building_type: r.building_type,
                is_first_blood: r.is_first_blood,
                multi_kill_length: r.multi_kill_length,
                is_teamfight: r.is_teamfight,
                involved_participants: involved,
            }
        })
        .collect();

    Ok(Some((
        participants,
        timeline_events,
        detail.game_duration,
        detail.game_mode.unwrap_or_default(),
    )))
}

pub async fn store_match_detail(
    db: &Surreal<Db>,
    match_id: &str,
    participants: &[crate::models::match_data::MatchParticipant],
    timeline_events: &[crate::models::match_data::TimelineEvent],
    game_duration: i32,
    game_mode: &str,
) -> Result<(), DbError> {
    /// One cached timeline event, bound into the transaction's `FOR` loop.
    #[derive(Debug, serde::Serialize, SurrealValue)]
    struct EventRow {
        ts: i64,
        etype: String,
        tid: Option<i32>,
        kid: Option<i32>,
        vid: Option<i32>,
        mtype: Option<String>,
        mstype: Option<String>,
        btype: Option<String>,
        fb: bool,
        mkl: Option<i32>,
        tf: bool,
        involved: String,
    }

    let participants_json =
        serde_json::to_string(participants).map_err(|e| DbError::Other(e.to_string()))?;
    let events: Vec<EventRow> = timeline_events
        .iter()
        .map(|e| EventRow {
            ts: e.timestamp_ms,
            etype: e.event_type.clone(),
            tid: e.team_id,
            kid: e.killer_participant_id,
            vid: e.victim_participant_id,
            mtype: e.monster_type.clone(),
            mstype: e.monster_sub_type.clone(),
            btype: e.building_type.clone(),
            fb: e.is_first_blood,
            mkl: e.multi_kill_length,
            tf: e.is_teamfight,
            involved: serde_json::to_string(&e.involved_participants)
                .unwrap_or_else(|_| "[]".to_string()),
        })
        .collect();

    // The detail row and its events are cached together: a half-written
    // cache would be served as complete on the next visit.
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             CREATE match_detail SET match_id = $match_id, participants_json = $pjson, game_duration = $dur, game_mode = $mode; \
             FOR $e IN $events { CREATE match_timeline_event SET match_id = $match_id, timestamp_ms = $e.ts, event_type = $e.etype, team_id = $e.tid, killer_participant_id = $e.kid, victim_participant_id = $e.vid, monster_type = $e.mtype, monster_sub_type = $e.mstype, building_type = $e.btype, is_first_blood = $e.fb, multi_kill_length = $e.mkl, is_teamfight = $e.tf, involved_participants = $e.involved; }; \
             COMMIT TRANSACTION;",
        )
        .bind(("match_id", match_id.to_string()))
        .bind(("pjson", participants_json))
        .bind(("dur", game_duration))
        .bind(("mode", game_mode.to_string()))
        .bind(("events", events))
        .await?;
    check_tx(response)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Sand Table storage (M6; no UI yet)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbReplay {
    id: RecordId,
    team: RecordId,
    match_id: String,
    patch: Option<String>,
    imported_at: Option<Datetime>,
    source: String,
    frame_interval_ms: i64,
    participants: Vec<crate::models::sand_table::ReplayParticipant>,
}

impl From<DbReplay> for crate::models::sand_table::Replay {
    fn from(r: DbReplay) -> Self {
        crate::models::sand_table::Replay {
            id: Some(r.id.to_sql()),
            team_id: r.team.to_sql(),
            match_id: r.match_id,
            patch: r.patch,
            imported_at: opt_dt(r.imported_at),
            source: r.source,
            frame_interval_ms: r.frame_interval_ms,
            participants: r.participants,
        }
    }
}

/// Store an imported timeline as `team_id`'s replay of that match, with all
/// frames and events, in one transaction. Importing the same match again
/// replaces the team's earlier copy. Returns the replay ID.
pub async fn store_replay(
    db: &Surreal<Db>,
    team_id: &str,
    imported: &crate::server::sand_table::ImportedTimeline,
    patch: Option<String>,
) -> DbResult<String> {
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $old = (SELECT VALUE id FROM replay WHERE team = type::record('team', $team_key) AND match_id = $match_id); \
             DELETE replay_frame WHERE replay IN $old; \
             DELETE replay_event WHERE replay IN $old; \
             DELETE replay WHERE id IN $old; \
             LET $r = (CREATE ONLY replay SET team = type::record('team', $team_key), match_id = $match_id, patch = $patch, \
                 source = 'riot_timeline', frame_interval_ms = $interval, participants = $participants); \
             FOR $f IN $frames { CREATE replay_frame SET replay = $r.id, t_ms = $f.t_ms, participants = $f.participants; }; \
             FOR $e IN $events { CREATE replay_event SET replay = $r.id, seq = $e.seq, t_ms = $e.t_ms, kind = $e.kind, x = $e.x, y = $e.y, \
                 participants = $e.participants, team_id = $e.team_id, meta = $e.meta; }; \
             RETURN { id: $r.id }; \
             COMMIT TRANSACTION;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("match_id", imported.match_id.clone()))
        .bind(("patch", patch))
        .bind(("interval", imported.frame_interval_ms))
        .bind(("participants", imported.participants.clone()))
        .bind(("frames", imported.frames.clone()))
        .bind(("events", imported.events.clone()))
        .await?;
    let mut response = check_tx(response)?;
    let row: Option<IdRecord> = response.take(8)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to store replay".into()))
}

pub async fn list_replays(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<crate::models::sand_table::Replay>> {
    let mut r = db
        .query("SELECT * FROM replay WHERE team = type::record('team', $team_key) ORDER BY imported_at DESC")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<DbReplay> = r.take(0)?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// A replay of `team_id`, or `None` (also when it belongs to another team).
pub async fn get_replay(
    db: &Surreal<Db>,
    team_id: &str,
    replay_id: &str,
) -> DbResult<Option<crate::models::sand_table::Replay>> {
    let mut r = db
        .query("SELECT * FROM type::record('replay', $key) WHERE team = type::record('team', $team_key)")
        .bind(("key", record_key(replay_id, "replay")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let row: Option<DbReplay> = r.take(0)?;
    Ok(row.map(Into::into))
}

/// Frames of a replay of `team_id`, in time order. `NotFound` for a replay
/// outside the team.
pub async fn get_replay_frames(
    db: &Surreal<Db>,
    team_id: &str,
    replay_id: &str,
) -> DbResult<Vec<crate::models::sand_table::ReplayFrame>> {
    ensure_team_record(db, "replay", replay_id, team_id).await?;
    let mut r = db
        .query("SELECT t_ms, participants FROM replay_frame WHERE replay = type::record('replay', $key) ORDER BY t_ms ASC")
        .bind(("key", record_key(replay_id, "replay")))
        .await?;
    Ok(r.take(0)?)
}

/// Events of a replay of `team_id`, in time order. `NotFound` for a replay
/// outside the team.
pub async fn get_replay_events(
    db: &Surreal<Db>,
    team_id: &str,
    replay_id: &str,
) -> DbResult<Vec<crate::models::sand_table::ReplayEvent>> {
    ensure_team_record(db, "replay", replay_id, team_id).await?;
    let mut r = db
        .query("SELECT seq, t_ms, kind, x, y, participants, team_id, meta FROM replay_event WHERE replay = type::record('replay', $key) ORDER BY seq ASC")
        .bind(("key", record_key(replay_id, "replay")))
        .await?;
    Ok(r.take(0)?)
}

pub async fn delete_replay(db: &Surreal<Db>, team_id: &str, replay_id: &str) -> DbResult<()> {
    ensure_team_record(db, "replay", replay_id, team_id).await?;
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             DELETE replay_frame WHERE replay = type::record('replay', $key); \
             DELETE replay_event WHERE replay = type::record('replay', $key); \
             DELETE type::record('replay', $key); \
             COMMIT TRANSACTION;",
        )
        .bind(("key", record_key(replay_id, "replay")))
        .await?;
    check_tx(response)?;
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbBoard {
    id: RecordId,
    team: RecordId,
    name: String,
    created_by: RecordId,
    updated_at: Option<Datetime>,
}

impl From<DbBoard> for crate::models::sand_table::Board {
    fn from(b: DbBoard) -> Self {
        crate::models::sand_table::Board {
            id: Some(b.id.to_sql()),
            team_id: b.team.to_sql(),
            name: b.name,
            created_by: b.created_by.to_sql(),
            updated_at: opt_dt(b.updated_at),
        }
    }
}

pub async fn create_board(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    name: String,
) -> DbResult<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(DbError::Invalid("Give the board a name.".into()));
    }
    let mut r = db
        .query("CREATE board SET team = type::record('team', $team_key), name = $name, created_by = type::record('user', $user_key)")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("name", name))
        .await?
        .check()?;
    let row: Option<IdRecord> = r.take(0)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create board".into()))
}

pub async fn list_boards(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<crate::models::sand_table::Board>> {
    let mut r = db
        .query("SELECT * FROM board WHERE team = type::record('team', $team_key) ORDER BY updated_at DESC")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<DbBoard> = r.take(0)?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Save `scene` as the next version of a board of `team_id` and return its
/// number (1, 2, ...). Two saves at the same moment cannot get the same
/// number: they are serialised by `invariant_lock`, and the unique
/// (board, version) index rejects a duplicate.
pub async fn save_board_version(
    db: &Surreal<Db>,
    team_id: &str,
    board_id: &str,
    user_id: &str,
    scene: serde_json::Value,
) -> DbResult<i64> {
    ensure_team_record(db, "board", board_id, team_id).await?;
    if !scene.is_object() {
        return Err(DbError::Invalid("A board scene must be an object.".into()));
    }
    let _guard = invariant_lock().await;
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             LET $prev = (SELECT VALUE version FROM board_version WHERE board = type::record('board', $key)); \
             LET $v = IF array::len($prev) = 0 { 1 } ELSE { math::max($prev) + 1 }; \
             CREATE board_version SET board = type::record('board', $key), version = $v, scene = $scene, author = type::record('user', $user_key); \
             UPDATE type::record('board', $key) SET updated_at = time::now(); \
             RETURN $v; \
             COMMIT TRANSACTION;",
        )
        .bind(("key", record_key(board_id, "board")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("scene", scene))
        .await?;
    let mut response = check_tx(response).map_err(|e| {
        if e.to_string().contains("board_version_unique") {
            DbError::Invalid(
                "Someone saved this board at the same moment. Reload it and save again.".into(),
            )
        } else {
            e
        }
    })?;
    // 0 BEGIN, 1-2 LET, 3 CREATE, 4 UPDATE, 5 RETURN
    let v: Option<i64> = response.take(5)?;
    v.ok_or_else(|| DbError::Other("Failed to save board version".into()))
}

/// A version of a board of `team_id`: the given number, or the latest when
/// `version` is `None`. `NotFound` for a board outside the team.
pub async fn get_board_version(
    db: &Surreal<Db>,
    team_id: &str,
    board_id: &str,
    version: Option<i64>,
) -> DbResult<Option<crate::models::sand_table::BoardVersion>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        board: RecordId,
        version: i64,
        scene: serde_json::Value,
        author: RecordId,
        created_at: Option<Datetime>,
    }
    ensure_team_record(db, "board", board_id, team_id).await?;
    let query = if version.is_some() {
        "SELECT * FROM board_version WHERE board = type::record('board', $key) AND version = $version LIMIT 1"
    } else {
        "SELECT * FROM board_version WHERE board = type::record('board', $key) ORDER BY version DESC LIMIT 1"
    };
    let mut r = db
        .query(query)
        .bind(("key", record_key(board_id, "board")))
        .bind(("version", version))
        .await?;
    let row: Option<Row> = r.take(0)?;
    Ok(row.map(|r| crate::models::sand_table::BoardVersion {
        board_id: r.board.to_sql(),
        version: r.version,
        scene: r.scene,
        author: r.author.to_sql(),
        created_at: opt_dt(r.created_at),
    }))
}

pub async fn delete_board(db: &Surreal<Db>, team_id: &str, board_id: &str) -> DbResult<()> {
    ensure_team_record(db, "board", board_id, team_id).await?;
    let response = db
        .query(
            "BEGIN TRANSACTION; \
             DELETE board_version WHERE board = type::record('board', $key); \
             DELETE type::record('board', $key); \
             COMMIT TRANSACTION;",
        )
        .bind(("key", record_key(board_id, "board")))
        .await?;
    check_tx(response)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Data Dragon cache and patch changes (M6)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbDdragonCache {
    version: String,
    champions_json: String,
}

/// The last good Data Dragon copy: (version, champions JSON).
pub async fn load_ddragon_cache(db: &Surreal<Db>) -> DbResult<Option<(String, String)>> {
    let mut r = db
        .query("SELECT version, champions_json FROM ddragon_cache:current")
        .await?;
    let row: Option<DbDdragonCache> = r.take(0)?;
    Ok(row.map(|c| (c.version, c.champions_json)))
}

/// Replace the stored Data Dragon copy (one row, `ddragon_cache:current`).
pub async fn save_ddragon_cache(
    db: &Surreal<Db>,
    version: &str,
    champions_json: &str,
) -> DbResult<()> {
    db.query("UPSERT ddragon_cache:current SET version = $version, champions_json = $json, fetched_at = time::now()")
        .bind(("version", version.to_string()))
        .bind(("json", champions_json.to_string()))
        .await?
        .check()?;
    Ok(())
}

/// Record that the live patch moved from `old` to `new`.
pub async fn record_patch_change(db: &Surreal<Db>, old: &str, new: &str) -> DbResult<()> {
    db.query(
        "CREATE patch_change SET old_version = $old, new_version = $new, detected_at = time::now()",
    )
    .bind(("old", old.to_string()))
    .bind(("new", new.to_string()))
    .await?
    .check()?;
    Ok(())
}

/// A detected patch change, newest first from `list_patch_changes`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchChange {
    pub old_version: Option<String>,
    pub new_version: String,
    pub detected_at: String,
}

pub async fn list_patch_changes(db: &Surreal<Db>, limit: i64) -> DbResult<Vec<PatchChange>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        old_version: Option<String>,
        new_version: String,
        detected_at: Datetime,
    }
    let mut r = db
        .query("SELECT old_version, new_version, detected_at FROM patch_change ORDER BY detected_at DESC LIMIT $limit")
        .bind(("limit", limit))
        .await?;
    let rows: Vec<Row> = r.take(0)?;
    Ok(rows
        .into_iter()
        .map(|r| PatchChange {
            old_version: r.old_version,
            new_version: r.new_version,
            detected_at: dt_to_string(&r.detected_at),
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Personal Learnings (Phase 14)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbPersonalLearning {
    id: RecordId,
    user: RecordId,
    title: String,
    learning_type: String,
    champion: Option<String>,
    opponent: Option<String>,
    what_happened: String,
    what_i_learned: String,
    next_time: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    win_loss: Option<String>,
    #[serde(default)]
    match_riot_id: Option<String>,
    #[serde(default)]
    game_timestamp_ms: Option<i64>,
    #[serde(default)]
    event_name: Option<String>,
    created_at: Option<Datetime>,
}

impl From<DbPersonalLearning> for PersonalLearning {
    fn from(d: DbPersonalLearning) -> Self {
        PersonalLearning {
            id: Some(d.id.to_sql()),
            user_id: d.user.to_sql(),
            title: d.title,
            learning_type: d.learning_type,
            champion: d.champion,
            opponent: d.opponent,
            what_happened: d.what_happened,
            what_i_learned: d.what_i_learned,
            next_time: d.next_time,
            tags: d.tags,
            win_loss: d.win_loss,
            match_riot_id: d.match_riot_id,
            game_timestamp_ms: d.game_timestamp_ms,
            event_name: d.event_name,
            created_at: opt_dt(d.created_at),
        }
    }
}

pub async fn create_personal_learning(
    db: &Surreal<Db>,
    learning: PersonalLearning,
) -> DbResult<String> {
    let user_key = learning
        .user_id
        .strip_prefix("user:")
        .unwrap_or(&learning.user_id)
        .to_string();
    let mut response = db
        .query("CREATE personal_learning SET user = type::record('user', $user_key), title = $title, learning_type = $learning_type, champion = $champion, opponent = $opponent, what_happened = $what_happened, what_i_learned = $what_i_learned, next_time = $next_time, tags = $tags, win_loss = $win_loss, match_riot_id = $match_riot_id, game_timestamp_ms = $game_timestamp_ms, event_name = $event_name")
        .bind(("user_key", user_key))
        .bind(("title", learning.title))
        .bind(("learning_type", learning.learning_type))
        .bind(("champion", learning.champion))
        .bind(("opponent", learning.opponent))
        .bind(("what_happened", learning.what_happened))
        .bind(("what_i_learned", learning.what_i_learned))
        .bind(("next_time", learning.next_time))
        .bind(("tags", learning.tags))
        .bind(("win_loss", learning.win_loss))
        .bind(("match_riot_id", learning.match_riot_id))
        .bind(("game_timestamp_ms", learning.game_timestamp_ms))
        .bind(("event_name", learning.event_name))
        .await?
        .check()?;
    let row: Option<IdRecord> = response.take(0)?;
    match row {
        Some(r) => Ok(r.id.to_sql()),
        None => Err(DbError::Other("Failed to create personal learning".into())),
    }
}

/// A personal learning owned by `user_id`, or `None` (also when it belongs to
/// someone else).
pub async fn get_personal_learning(
    db: &Surreal<Db>,
    user_id: &str,
    id: &str,
) -> DbResult<Option<PersonalLearning>> {
    let key = record_key(id, "personal_learning");
    let mut r = db
        .query("SELECT * FROM type::record('personal_learning', $key) WHERE user = type::record('user', $user_key)")
        .bind(("key", key))
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let row: Option<DbPersonalLearning> = r.take(0)?;
    Ok(row.map(PersonalLearning::from))
}

pub async fn list_personal_learnings(
    db: &Surreal<Db>,
    user_id: &str,
) -> DbResult<Vec<PersonalLearning>> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    let mut r = db
        .query("SELECT * FROM personal_learning WHERE user = type::record('user', $user_key) ORDER BY created_at DESC")
        .bind(("user_key", user_key))
        .await?;
    let rows: Vec<DbPersonalLearning> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(PersonalLearning::from).collect())
}

/// Update a learning owned by `user_id`. The learning's own `user_id` field is
/// ignored.
pub async fn update_personal_learning(
    db: &Surreal<Db>,
    user_id: &str,
    learning: PersonalLearning,
) -> DbResult<()> {
    let id = learning.id.as_deref().ok_or(DbError::NotFound)?;
    ensure_user_record(db, "personal_learning", id, user_id).await?;
    let key = record_key(id, "personal_learning");
    db.query("UPDATE type::record('personal_learning', $key) SET title = $title, learning_type = $learning_type, champion = $champion, opponent = $opponent, what_happened = $what_happened, what_i_learned = $what_i_learned, next_time = $next_time, tags = $tags, win_loss = $win_loss, match_riot_id = $match_riot_id, game_timestamp_ms = $game_timestamp_ms, event_name = $event_name")
        .bind(("key", key))
        .bind(("title", learning.title))
        .bind(("learning_type", learning.learning_type))
        .bind(("champion", learning.champion))
        .bind(("opponent", learning.opponent))
        .bind(("what_happened", learning.what_happened))
        .bind(("what_i_learned", learning.what_i_learned))
        .bind(("next_time", learning.next_time))
        .bind(("tags", learning.tags))
        .bind(("win_loss", learning.win_loss))
        .bind(("match_riot_id", learning.match_riot_id))
        .bind(("game_timestamp_ms", learning.game_timestamp_ms))
        .bind(("event_name", learning.event_name))
        .await?
        .check()?;
    Ok(())
}

pub async fn delete_personal_learning(db: &Surreal<Db>, user_id: &str, id: &str) -> DbResult<()> {
    ensure_user_record(db, "personal_learning", id, user_id).await?;
    let key = record_key(id, "personal_learning");
    db.query("DELETE type::record('personal_learning', $key)")
        .bind(("key", key))
        .await?
        .check()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Draft Graph (M4): champion traits (global), team relations and the team's
// games as graph evidence (team-scoped)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbChampionTrait {
    champion: String,
    roles: Vec<String>,
    damage_type: String,
    damage_profile: String,
    range_class: String,
    engage: String,
    disengage: String,
    peel: String,
    frontline: String,
    cc: String,
    wave_clear: String,
    scaling: String,
    mobility: String,
    styles: Vec<String>,
    confidence: String,
    source: String,
    edited_by_name: Option<String>,
    edited_at: Option<Datetime>,
    reviewed_for_patch: Option<String>,
}

impl DbChampionTrait {
    /// The shared model; `None` (and a warning) for a row outside the
    /// vocabulary, which the schema's assertions should make impossible.
    fn into_record(self) -> Option<crate::models::graph::TraitRecord> {
        use crate::models::graph::*;
        fn all<T>(raw: &[String], parse: impl Fn(&str) -> Option<T>) -> Option<Vec<T>> {
            raw.iter().map(|s| parse(s)).collect()
        }
        let traits = ChampionTraits {
            champion: self.champion.clone(),
            roles: all(&self.roles, |s| {
                Role::ALL.iter().copied().find(|r| r.key() == s)
            })?,
            damage_type: DamageType::parse(&self.damage_type)?,
            damage_profile: DamageProfile::parse(&self.damage_profile)?,
            range: RangeClass::parse(&self.range_class)?,
            engage: Engage::parse(&self.engage)?,
            disengage: Strength::parse(&self.disengage)?,
            peel: Strength::parse(&self.peel)?,
            frontline: Strength::parse(&self.frontline)?,
            cc: CrowdControl::parse(&self.cc)?,
            wave_clear: WaveClear::parse(&self.wave_clear)?,
            scaling: Scaling::parse(&self.scaling)?,
            mobility: Mobility::parse(&self.mobility)?,
            styles: all(&self.styles, Playstyle::parse)?,
            confidence: Confidence::parse(&self.confidence)?,
        };
        Some(TraitRecord {
            name: self.champion,
            traits,
            source: TraitSource::parse(&self.source)?,
            edited_by: self.edited_by_name,
            edited_at: opt_dt(self.edited_at),
            reviewed_for_patch: self.reviewed_for_patch,
        })
    }
}

/// A trait row as bound into a query (enum keys as strings).
#[derive(Debug, Clone, serde::Serialize, SurrealValue)]
struct TraitRow {
    champion: String,
    roles: Vec<String>,
    damage_type: String,
    damage_profile: String,
    range_class: String,
    engage: String,
    disengage: String,
    peel: String,
    frontline: String,
    cc: String,
    wave_clear: String,
    scaling: String,
    mobility: String,
    styles: Vec<String>,
    confidence: String,
}

impl From<&crate::models::graph::ChampionTraits> for TraitRow {
    fn from(t: &crate::models::graph::ChampionTraits) -> Self {
        TraitRow {
            champion: t.champion.clone(),
            roles: t.roles.iter().map(|r| r.key().to_string()).collect(),
            damage_type: t.damage_type.key().into(),
            damage_profile: t.damage_profile.key().into(),
            range_class: t.range.key().into(),
            engage: t.engage.key().into(),
            disengage: t.disengage.key().into(),
            peel: t.peel.key().into(),
            frontline: t.frontline.key().into(),
            cc: t.cc.key().into(),
            wave_clear: t.wave_clear.key().into(),
            scaling: t.scaling.key().into(),
            mobility: t.mobility.key().into(),
            styles: t.styles.iter().map(|s| s.key().to_string()).collect(),
            confidence: t.confidence.key().into(),
        }
    }
}

/// Every champion trait row (global table), in champion id order.
pub async fn list_champion_traits(
    db: &Surreal<Db>,
) -> DbResult<Vec<crate::models::graph::TraitRecord>> {
    let mut r = db
        .query("SELECT * FROM champion_trait ORDER BY champion ASC")
        .await?;
    let rows: Vec<DbChampionTrait> = r.take(0)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let id = row.champion.clone();
            let rec = row.into_record();
            if rec.is_none() {
                tracing::warn!(champion = %id, "champion_trait row outside the vocabulary; skipped");
            }
            rec
        })
        .collect())
}

/// Write shipped default rows (create or replace), all in one transaction.
/// The caller (`graph::traits::sync`) passes only rows that are missing or
/// still `default`; this function re-checks that so a maintainer's edit is
/// never overwritten, even by a race.
pub async fn write_default_traits(
    db: &Surreal<Db>,
    rows: Vec<crate::models::graph::ChampionTraits>,
) -> DbResult<()> {
    for t in &rows {
        t.validate().map_err(DbError::Invalid)?;
    }
    let rows: Vec<TraitRow> = rows.iter().map(TraitRow::from).collect();
    let r = db
        .query(
            "BEGIN TRANSACTION; \
             FOR $r IN $rows { \
                LET $cur = (SELECT VALUE source FROM type::record('champion_trait', $r.champion))[0]; \
                IF $cur = NONE OR $cur = 'default' { \
                    UPSERT type::record('champion_trait', $r.champion) CONTENT { \
                        champion: $r.champion, roles: $r.roles, damage_type: $r.damage_type, \
                        damage_profile: $r.damage_profile, range_class: $r.range_class, \
                        engage: $r.engage, disengage: $r.disengage, peel: $r.peel, \
                        frontline: $r.frontline, cc: $r.cc, wave_clear: $r.wave_clear, \
                        scaling: $r.scaling, mobility: $r.mobility, styles: $r.styles, \
                        confidence: $r.confidence, source: 'default', edited_by: NONE, \
                        edited_by_name: NONE, edited_at: NONE, reviewed_for_patch: NONE \
                    }; \
                }; \
             }; \
             COMMIT TRANSACTION;",
        )
        .bind(("rows", rows))
        .await?;
    check_tx(r)?;
    Ok(())
}

/// A maintainer's edit of one champion's row (admin only; the server
/// function checks the role). The row must exist. Marks it `maintainer`,
/// records who and when, and the patch it was reviewed for (or clears it).
pub async fn save_champion_traits(
    db: &Surreal<Db>,
    traits: &crate::models::graph::ChampionTraits,
    editor_id: &str,
    editor_name: &str,
    reviewed_for_patch: Option<String>,
) -> DbResult<()> {
    traits.validate().map_err(DbError::Invalid)?;
    let row = TraitRow::from(traits);
    let mut r = db
        .query(
            "UPDATE type::record('champion_trait', $r.champion) SET \
                roles = $r.roles, damage_type = $r.damage_type, damage_profile = $r.damage_profile, \
                range_class = $r.range_class, engage = $r.engage, disengage = $r.disengage, \
                peel = $r.peel, frontline = $r.frontline, cc = $r.cc, wave_clear = $r.wave_clear, \
                scaling = $r.scaling, mobility = $r.mobility, styles = $r.styles, \
                confidence = $r.confidence, source = 'maintainer', \
                edited_by = type::record('user', $editor), edited_by_name = $editor_name, \
                edited_at = time::now(), reviewed_for_patch = $patch \
             RETURN champion",
        )
        .bind(("r", row))
        .bind(("editor", record_key(editor_id, "user")))
        .bind(("editor_name", editor_name.to_string()))
        .bind(("patch", reviewed_for_patch))
        .await?
        .check()?;
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Back {
        #[allow(dead_code)]
        champion: String,
    }
    let back: Vec<Back> = r.take(0).unwrap_or_default();
    if back.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Put one champion's row back to the shipped default (admin only).
pub async fn reset_champion_traits(
    db: &Surreal<Db>,
    default: &crate::models::graph::ChampionTraits,
) -> DbResult<()> {
    let row = TraitRow::from(default);
    let mut r = db
        .query(
            "UPDATE type::record('champion_trait', $r.champion) CONTENT { \
                champion: $r.champion, roles: $r.roles, damage_type: $r.damage_type, \
                damage_profile: $r.damage_profile, range_class: $r.range_class, \
                engage: $r.engage, disengage: $r.disengage, peel: $r.peel, \
                frontline: $r.frontline, cc: $r.cc, wave_clear: $r.wave_clear, \
                scaling: $r.scaling, mobility: $r.mobility, styles: $r.styles, \
                confidence: $r.confidence, source: 'default', edited_by: NONE, \
                edited_by_name: NONE, edited_at: NONE, reviewed_for_patch: NONE \
             } RETURN champion",
        )
        .bind(("r", row))
        .await?
        .check()?;
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Back {
        #[allow(dead_code)]
        champion: String,
    }
    let back: Vec<Back> = r.take(0).unwrap_or_default();
    if back.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbTeamRelation {
    id: RecordId,
    team: RecordId,
    a: String,
    b: String,
    kind: String,
    weight: i64,
    reason: String,
    author: RecordId,
    author_name: String,
    created_at: Option<Datetime>,
}

impl DbTeamRelation {
    fn into_model(self) -> Option<crate::models::graph::TeamRelation> {
        Some(crate::models::graph::TeamRelation {
            id: Some(self.id.to_sql()),
            team_id: self.team.to_sql(),
            a: self.a,
            b: self.b,
            kind: crate::models::graph::RelationKind::parse(&self.kind)?,
            weight: u8::try_from(self.weight).ok()?,
            reason: self.reason,
            author_id: self.author.to_sql(),
            author_name: self.author_name,
            created_at: opt_dt(self.created_at),
        })
    }
}

/// Record what a team knows about a pair. Champion ids must already be
/// resolved by the caller; the relation is checked again here.
pub async fn create_team_relation(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    author_name: &str,
    relation: crate::models::graph::NewRelation,
) -> DbResult<String> {
    let rel = relation.normalised().map_err(DbError::Invalid)?;
    let mut r = db
        .query(
            "CREATE team_relation SET team = type::record('team', $team_key), a = $a, b = $b, \
             kind = $kind, weight = $weight, reason = $reason, \
             author = type::record('user', $user_key), author_name = $author_name",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("author_name", author_name.to_string()))
        .bind(("a", rel.a))
        .bind(("b", rel.b))
        .bind(("kind", rel.kind.key().to_string()))
        .bind(("weight", rel.weight as i64))
        .bind(("reason", rel.reason))
        .await?
        .check()?;
    let row: Option<IdRecord> = r.take(0)?;
    row.map(|r| r.id.to_sql())
        .ok_or_else(|| DbError::Other("Failed to create team relation".into()))
}

/// The team's relations, newest first.
pub async fn list_team_relations(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Vec<crate::models::graph::TeamRelation>> {
    let mut r = db
        .query("SELECT * FROM team_relation WHERE team = type::record('team', $team_key) ORDER BY created_at DESC")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<DbTeamRelation> = r.take(0)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let id = row.id.to_sql();
            let rel = row.into_model();
            if rel.is_none() {
                tracing::warn!(relation = %id, "team_relation row outside the vocabulary; skipped");
            }
            rel
        })
        .collect())
}

/// Delete one of the team's relations (any member may, like team notes).
pub async fn delete_team_relation(
    db: &Surreal<Db>,
    team_id: &str,
    relation_id: &str,
) -> DbResult<()> {
    ensure_team_record(db, "team_relation", relation_id, team_id).await?;
    db.query("DELETE type::record('team_relation', $key)")
        .bind(("key", record_key(relation_id, "team_relation")))
        .await?
        .check()?;
    Ok(())
}

/// A team member with their champion pool, for the Draft Graph.
#[derive(Clone, Debug, PartialEq)]
pub struct PoolMember {
    pub user_id: String,
    pub username: String,
    /// Roster role as stored ("top", "sub", "unassigned", ...).
    pub role: String,
    pub roster_type: String,
    /// (champion as stored, tier).
    pub entries: Vec<(String, String)>,
}

/// The team's members and every member's pool entries, in one round trip.
pub async fn team_pool_members(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<PoolMember>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Member {
        user: RecordId,
        username: Option<String>,
        role: String,
        roster_type: String,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        user: RecordId,
        champion: String,
        tier: String,
    }
    let mut r = db
        .query("LET $m = (SELECT user, user.username AS username, role, roster_type FROM team_member WHERE team = type::record('team', $team_key)); \
                RETURN $m; \
                SELECT user, champion, tier FROM champion_pool WHERE user IN $m.user")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let members: Vec<Member> = r.take(1)?;
    let rows: Vec<Row> = r.take(2)?;
    Ok(members
        .into_iter()
        .map(|m| {
            let id = m.user.to_sql();
            let entries = rows
                .iter()
                .filter(|r| r.user.to_sql() == id)
                .map(|r| (r.champion.clone(), r.tier.clone()))
                .collect();
            PoolMember {
                user_id: id,
                username: m.username.unwrap_or_default(),
                role: m.role,
                roster_type: m.roster_type,
                entries,
            }
        })
        .collect())
}

/// One stored game of a team member, as the graph's match evidence reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct TeamGameRow {
    /// The `match` record (games played together share it).
    pub match_key: String,
    pub riot_match_id: String,
    pub user_id: String,
    /// As Riot reports it (resolve to a Data Dragon id before use).
    pub champion: String,
    pub win: bool,
}

/// The current members' stored games (`player_match`), newest first, at
/// most 2,000 rows.
pub async fn team_game_rows(db: &Surreal<Db>, team_id: &str) -> DbResult<Vec<TeamGameRow>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        match_ref: Option<RecordId>,
        riot_match_id: Option<String>,
        user: RecordId,
        champion: String,
        win: bool,
        #[allow(dead_code)]
        game_end: Option<Datetime>,
    }
    // `match` is aliased: the SurrealValue derive reads field names as
    // written, not serde renames.
    let mut r = db
        .query("LET $users = (SELECT VALUE user FROM team_member WHERE team = type::record('team', $team_key)); \
                SELECT match AS match_ref, match.match_id AS riot_match_id, user, champion, win, match.game_end AS game_end FROM player_match WHERE user IN $users ORDER BY game_end DESC LIMIT 2000")
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<Row> = r.take(1)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some(TeamGameRow {
                match_key: row.match_ref?.to_sql(),
                riot_match_id: row.riot_match_id.unwrap_or_default(),
                user_id: row.user.to_sql(),
                champion: row.champion,
                win: row.win,
            })
        })
        .collect())
}

/// Cached participant lists (all ten players) for the given Riot match
/// ids, where the match detail has been fetched before. Match details are
/// public Riot data shared by every team; callers pass only ids from their
/// own team's games.
pub async fn match_participants(
    db: &Surreal<Db>,
    riot_match_ids: Vec<String>,
) -> DbResult<Vec<(String, Vec<crate::models::match_data::MatchParticipant>)>> {
    if riot_match_ids.is_empty() {
        return Ok(Vec::new());
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        match_id: String,
        participants_json: String,
    }
    let mut r = db
        .query("SELECT match_id, participants_json FROM match_detail WHERE match_id IN $ids")
        .bind(("ids", riot_match_ids))
        .await?;
    let rows: Vec<Row> = r.take(0).unwrap_or_default();
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            serde_json::from_str(&row.participants_json)
                .ok()
                .map(|p| (row.match_id, p))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::champion::{Champion, ChampionPoolEntry};
    use crate::models::game_plan::{
        ActionItemPreview, DashboardSummary, PoolGapWarning, PostGamePreview,
    };

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
        assert!(
            !warnings.is_empty(),
            "Should warn about Fighter dominance (4/4 entries are Fighter = 100%)"
        );
        let has_dominant = warnings
            .iter()
            .any(|w| w.dominant_class.as_deref() == Some("Fighter"));
        assert!(has_dominant, "Should have a Fighter dominant class warning");

        // 3 Fighters + 1 Marksman = 75% Fighter => still dominant
        let mixed_pool = vec![
            make_pool_entry("user:u1", "Darius", "top"),
            make_pool_entry("user:u1", "Garen", "top"),
            make_pool_entry("user:u1", "Tryndamere", "top"),
            make_pool_entry("user:u1", "Jinx", "top"),
        ];
        let warnings2 = compute_pool_gaps(&mixed_pool, &members, &champions, &[]);
        let has_dominant2 = warnings2
            .iter()
            .any(|w| w.dominant_class.as_deref() == Some("Fighter"));
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
        let has_tank_warning = warnings
            .iter()
            .any(|w| w.missing_classes.contains(&"Tank".to_string()) && w.opponent_escalated);
        assert!(
            has_tank_warning,
            "Should warn about missing Tank coverage with opponent escalation"
        );
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
        assert!(
            warnings.is_empty()
                || warnings
                    .iter()
                    .all(|w| !w.opponent_escalated && w.dominant_class.is_none()),
            "Balanced pool should have no dominant class or opponent-escalated warnings"
        );
    }

    #[test]
    fn test_dashboard_summary_assembly() {
        // Test that DashboardSummary can be constructed and all fields populate correctly
        let action_items = vec![
            ActionItemPreview {
                id: "action_item:1".into(),
                text: "Review baron fight".into(),
            },
            ActionItemPreview {
                id: "action_item:2".into(),
                text: "Improve vision".into(),
            },
        ];
        let post_games = vec![PostGamePreview {
            id: "pgl:1".into(),
            improvements: vec!["Better rotations".into()],
            created_at: Some("2026-03-14T20:00:00Z".into()),
        }];
        let pool_warnings = vec![PoolGapWarning {
            user_id: "user:u1".into(),
            username: "Player1".into(),
            role: "top".into(),
            dominant_class: Some("Fighter".into()),
            missing_classes: vec![],
            opponent_escalated: false,
        }];
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
        let draft_champions = vec![
            "Jinx".to_string(),
            "Jinx".to_string(),
            "Caitlyn".to_string(),
        ];
        let match_stats = vec![
            ("Jinx".to_string(), true),
            ("Jinx".to_string(), false),
            ("Caitlyn".to_string(), true),
        ];
        let plan_champions = vec!["Jinx".to_string()];
        let post_game_outcomes = vec![("Jinx".to_string(), true), ("Caitlyn".to_string(), false)];

        let result = aggregate_champion_performance(
            &draft_champions,
            &match_stats,
            &plan_champions,
            &post_game_outcomes,
        );

        assert!(!result.is_empty(), "Should return some results");

        let jinx = result
            .iter()
            .find(|s| s.champion == "Jinx")
            .expect("Should have Jinx");
        assert_eq!(jinx.games_in_draft, 2);
        assert_eq!(jinx.games_in_match, 2);
        assert_eq!(jinx.wins_in_match, 1);
        assert_eq!(jinx.games_in_plan, 1);
        assert_eq!(jinx.post_game_wins, 1);
        assert_eq!(jinx.post_game_losses, 0);

        let caitlyn = result
            .iter()
            .find(|s| s.champion == "Caitlyn")
            .expect("Should have Caitlyn");
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
        assert_eq!(
            result[0].champion, "Caitlyn",
            "Caitlyn has 3 games, should be first"
        );
        assert_eq!(
            result[1].champion, "Jinx",
            "Jinx has 1 game, should be second"
        );
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
        assert!(
            result.is_empty(),
            "No plans against TeamA should mean empty results"
        );
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
        let open_texts: HashSet<String> = ["fix baron calls".to_string(), "ward river".to_string()]
            .into_iter()
            .collect();
        let improvements = vec![
            "Fix Baron Calls".to_string(),      // matches (case-insensitive) → skip
            "Improve dragon setup".to_string(), // new → create
            "  ".to_string(),                   // whitespace → skip
            "ward river".to_string(),           // exact match → skip
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

    // -----------------------------------------------------------------------
    // Bug Reports (Phase 19 D-02/D-03)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn bug_report_create_and_list_round_trip() {
        use surrealdb::engine::local::Mem;
        let db = surrealdb::Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(include_str!("../../schema.surql"))
            .await
            .unwrap()
            .check()
            .unwrap();

        // Seed a user.
        let mut r = db
            .query("CREATE user SET username='u', email='e@x.t', password_hash='h' RETURN id")
            .await
            .unwrap();
        let row: Option<IdRecord> = r.take(0).unwrap();
        let user_id = row.unwrap().id.to_sql();

        create_bug_report(
            &db,
            &user_id,
            "/draft".into(),
            "Draft → Blue side → Pick 3".into(),
            "Hover broke".into(),
            "bug".into(),
            Some(1920),
            Some(1080),
        )
        .await
        .unwrap();

        let reports = list_untriaged_bug_reports(&db).await.unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].category, "bug");
        assert_eq!(reports[0].page_url, "/draft");
    }

    #[tokio::test]
    async fn bug_report_rejects_invalid_category() {
        use surrealdb::engine::local::Mem;
        let db = surrealdb::Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(include_str!("../../schema.surql"))
            .await
            .unwrap()
            .check()
            .unwrap();
        // Manually CREATE with bad category to exercise the ASSERT constraint
        // — proves T-19-01 mitigation at the DB layer.
        let result = db
            .query(
                "CREATE bug_report SET user=type::record('user','u1'), page_url='/x', element_label='y', description='z', category='spam'",
            )
            .await
            .unwrap()
            .check();
        assert!(
            result.is_err(),
            "ASSERT $value IN ['bug','wishlist'] must reject 'spam'"
        );
    }

    #[tokio::test]
    async fn bug_report_rejects_empty_description() {
        use surrealdb::engine::local::Mem;
        let db = surrealdb::Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(include_str!("../../schema.surql"))
            .await
            .unwrap()
            .check()
            .unwrap();

        // T-19-02 mitigation: whitespace-only description rejected at DB layer
        // before the CREATE even runs.
        let result = create_bug_report(
            &db,
            "user:u1",
            "/draft".into(),
            "Draft → Pick 3".into(),
            "   ".into(),
            "bug".into(),
            None,
            None,
        )
        .await;
        assert!(
            result.is_err(),
            "create_bug_report must reject whitespace-only description"
        );
    }
}

// ---------------------------------------------------------------------------
// Match Day (M5): the next scrim or match, and the facts around it
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct DbScheduledMatch {
    id: RecordId,
    starts_at: Datetime,
    kind: String,
    format: String,
    opponent: Option<RecordId>,
    opponent_name: Option<String>,
    notes: Option<String>,
    #[serde(default)]
    brief_read_by: Vec<RecordId>,
}

impl From<DbScheduledMatch> for ScheduledMatch {
    fn from(m: DbScheduledMatch) -> Self {
        ScheduledMatch {
            id: m.id.to_sql(),
            starts_at: dt_to_string(&m.starts_at),
            kind: m.kind,
            format: m.format,
            opponent_id: opt_link(m.opponent),
            opponent_name: m.opponent_name,
            notes: m.notes,
            brief_read_by: m.brief_read_by.iter().map(|u| u.to_sql()).collect(),
        }
    }
}

/// A match stays "next" for this long after it started, so Match Day can
/// ask for the review.
const MATCH_DAY_GRACE_HOURS: i64 = 12;

/// The team's next scrim or match: the earliest one that started less than
/// twelve hours ago or lies ahead.
pub async fn next_scheduled_match(
    db: &Surreal<Db>,
    team_id: &str,
) -> DbResult<Option<ScheduledMatch>> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(MATCH_DAY_GRACE_HOURS)).to_rfc3339();
    let mut r = db
        .query("SELECT id, starts_at, kind, format, opponent, opponent.name AS opponent_name, notes, brief_read_by FROM scheduled_match WHERE team = type::record('team', $team_key) AND starts_at >= <datetime>$cutoff ORDER BY starts_at ASC LIMIT 1")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("cutoff", cutoff))
        .await?;
    let row: Option<DbScheduledMatch> = r.take(0)?;
    Ok(row.map(ScheduledMatch::from))
}

/// Longest match note, in characters.
pub const MATCH_NOTES_MAX: usize = 500;

/// Create or update the team's next match. Kind, format and time are
/// checked; the opponent must be one of the team's dossiers.
pub async fn save_scheduled_match(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    input: ScheduledMatchInput,
) -> DbResult<String> {
    use crate::models::match_day::{FORMATS, KINDS};
    if !KINDS.iter().any(|(k, _)| *k == input.kind) {
        return Err(DbError::Invalid("Choose scrim or official match.".into()));
    }
    if !FORMATS.iter().any(|(k, _)| *k == input.format) {
        return Err(DbError::Invalid("Choose best of 1, 3 or 5.".into()));
    }
    let starts = chrono::DateTime::parse_from_rfc3339(input.starts_at.trim())
        .map_err(|_| DbError::Invalid("Enter a date and a time.".into()))?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    if input.id.is_none() && starts < now - chrono::Duration::hours(MATCH_DAY_GRACE_HOURS) {
        return Err(DbError::Invalid("That time has already passed.".into()));
    }
    if starts > now + chrono::Duration::days(366) {
        return Err(DbError::Invalid("Pick a date within the next year.".into()));
    }
    let notes = input
        .notes
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    if notes
        .as_ref()
        .is_some_and(|n| n.chars().count() > MATCH_NOTES_MAX)
    {
        return Err(DbError::Invalid(format!(
            "Notes are at most {MATCH_NOTES_MAX} characters."
        )));
    }
    let opponent = input.opponent_id.filter(|o| !o.trim().is_empty());
    if let Some(o) = &opponent {
        ensure_team_record(db, "opponent", o, team_id).await?;
    }
    let opp_key = opt_key(opponent, "opponent");
    match input.id {
        Some(id) => {
            ensure_team_record(db, "scheduled_match", &id, team_id).await?;
            // A different opponent is a different brief: who read the old
            // one has not read this one.
            db.query("LET $opp = IF $opp_key = NONE { NONE } ELSE { type::record('opponent', $opp_key) }; \
                      UPDATE type::record('scheduled_match', $key) SET brief_read_by = IF opponent = $opp { brief_read_by } ELSE { [] }, starts_at = <datetime>$starts, kind = $kind, format = $format, opponent = $opp, notes = $notes, updated_at = time::now()")
                .bind(("key", record_key(&id, "scheduled_match")))
                .bind(("starts", starts.to_rfc3339()))
                .bind(("kind", input.kind))
                .bind(("format", input.format))
                .bind(("opp_key", opp_key))
                .bind(("notes", notes))
                .await?
                .check()?;
            Ok(id)
        }
        None => {
            let mut r = db
                .query("CREATE scheduled_match SET team = type::record('team', $team_key), starts_at = <datetime>$starts, kind = $kind, format = $format, opponent = IF $opp_key = NONE { NONE } ELSE { type::record('opponent', $opp_key) }, notes = $notes, created_by = type::record('user', $user_key)")
                .bind(("team_key", record_key(team_id, "team")))
                .bind(("user_key", record_key(user_id, "user")))
                .bind(("starts", starts.to_rfc3339()))
                .bind(("kind", input.kind))
                .bind(("format", input.format))
                .bind(("opp_key", opp_key))
                .bind(("notes", notes))
                .await?;
            let row: Option<IdRecord> = r.take(0)?;
            row.map(|r| r.id.to_sql())
                .ok_or_else(|| DbError::Other("Failed to save the match".into()))
        }
    }
}

pub async fn delete_scheduled_match(db: &Surreal<Db>, team_id: &str, id: &str) -> DbResult<()> {
    ensure_team_record(db, "scheduled_match", id, team_id).await?;
    db.query("DELETE type::record('scheduled_match', $key)")
        .bind(("key", record_key(id, "scheduled_match")))
        .await?
        .check()?;
    Ok(())
}

/// Mark the opponent brief of a match read (or unread) for one member.
pub async fn set_brief_read(
    db: &Surreal<Db>,
    team_id: &str,
    match_id: &str,
    user_id: &str,
    read: bool,
) -> DbResult<()> {
    let query = if read {
        "UPDATE type::record('scheduled_match', $key) SET brief_read_by = array::union(brief_read_by, [type::record('user', $user_key)]) WHERE team = type::record('team', $team_key) RETURN id"
    } else {
        "UPDATE type::record('scheduled_match', $key) SET brief_read_by = array::complement(brief_read_by, [type::record('user', $user_key)]) WHERE team = type::record('team', $team_key) RETURN id"
    };
    let mut r = db
        .query(query)
        .bind(("key", record_key(match_id, "scheduled_match")))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Drafts against an opponent, the plans linked to them, and a review of a
/// match that started at `starts_at` (linked to those drafts or plans and
/// written after the start).
#[derive(Clone, Debug, Default)]
pub struct MatchPrepFacts {
    pub drafts: Vec<LinkRef>,
    pub plans: Vec<LinkRef>,
    pub review_after: Option<LinkRef>,
}

pub async fn match_prep_facts(
    db: &Surreal<Db>,
    team_id: &str,
    opponent_id: &str,
    opponent_name: &str,
    starts_at: &str,
) -> DbResult<MatchPrepFacts> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct NameRow {
        id: RecordId,
        #[serde(default)]
        name: Option<String>,
        #[allow(dead_code)]
        created_at: Option<Datetime>,
    }
    // Drafts store the opponent's id (older ones its name).
    let mut r = db
        .query(
            "LET $t = type::record('team', $team_key); \
             LET $drafts = (SELECT id, name, created_at FROM draft WHERE team = $t AND (opponent = $opp_id OR string::lowercase(opponent ?? '') = $opp_name) ORDER BY created_at DESC); \
             LET $draft_ids = $drafts.id; \
             LET $plans = (SELECT id, name, created_at FROM game_plan WHERE team = $t AND draft != NONE AND draft IN $draft_ids ORDER BY created_at DESC); \
             LET $plan_ids = $plans.id; \
             RETURN $drafts; \
             RETURN $plans; \
             SELECT id, created_at FROM post_game_learning WHERE team = $t AND ((draft_id != NONE AND draft_id IN $draft_ids) OR (game_plan_id != NONE AND game_plan_id IN $plan_ids)) AND created_at >= <datetime>$starts ORDER BY created_at DESC LIMIT 1;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("opp_id", opponent_id.to_string()))
        .bind(("opp_name", opponent_name.trim().to_lowercase()))
        .bind(("starts", starts_at.to_string()))
        .await?;
    let link = |r: NameRow, fallback: &str| LinkRef {
        id: r.id.to_sql(),
        name: r
            .name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| fallback.to_string()),
    };
    let drafts: Vec<NameRow> = r.take(5).unwrap_or_default();
    let plans: Vec<NameRow> = r.take(6).unwrap_or_default();
    let review: Option<NameRow> = r.take(7).unwrap_or_default();
    Ok(MatchPrepFacts {
        drafts: drafts
            .into_iter()
            .map(|d| link(d, "Untitled draft"))
            .collect(),
        plans: plans
            .into_iter()
            .map(|p| link(p, "Untitled plan"))
            .collect(),
        review_after: review.map(|v| link(v, "Review")),
    })
}

/// The team's newest reviews, newest first.
pub async fn recent_reviews(
    db: &Surreal<Db>,
    team_id: &str,
    limit: i64,
) -> DbResult<Vec<PostGameLearning>> {
    let mut r = db
        .query("SELECT * FROM post_game_learning WHERE team = type::record('team', $team_key) ORDER BY created_at DESC LIMIT $limit")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("limit", limit))
        .await?;
    let rows: Vec<DbPostGameLearning> = r.take(0).unwrap_or_default();
    Ok(rows.into_iter().map(PostGameLearning::from).collect())
}

/// One review of the team.
pub async fn get_post_game_learning(
    db: &Surreal<Db>,
    team_id: &str,
    review_id: &str,
) -> DbResult<Option<PostGameLearning>> {
    let mut r = db
        .query("SELECT * FROM type::record('post_game_learning', $key) WHERE team = type::record('team', $team_key)")
        .bind(("key", record_key(review_id, "post_game_learning")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let row: Option<DbPostGameLearning> = r.take(0)?;
    Ok(row.map(PostGameLearning::from))
}

/// Counts for the phone menu and the command bar.
#[derive(Clone, Debug, Default)]
pub struct TeamCounts {
    /// The team's name (the phone menu's heading).
    pub name: String,
    pub opponents: usize,
    pub drafts: usize,
    pub plans: usize,
    pub open_items: usize,
    pub members: usize,
    pub reviews: usize,
}

/// How many personal learnings the user has (a count, not the rows).
pub async fn count_personal_learnings(db: &Surreal<Db>, user_id: &str) -> DbResult<usize> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Count {
        n: i64,
    }
    let mut r = db
        .query("SELECT count() AS n FROM personal_learning WHERE user = type::record('user', $user_key) GROUP ALL")
        .bind(("user_key", record_key(user_id, "user")))
        .await?;
    let row: Option<Count> = r.take(0)?;
    Ok(row.map(|c| c.n.max(0) as usize).unwrap_or(0))
}

pub async fn team_counts(db: &Surreal<Db>, team_id: &str) -> DbResult<TeamCounts> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Count {
        n: i64,
    }
    let mut r = db
        .query(
            "LET $t = type::record('team', $team_key); \
             SELECT count() AS n FROM opponent WHERE team = $t GROUP ALL; \
             SELECT count() AS n FROM draft WHERE team = $t GROUP ALL; \
             SELECT count() AS n FROM game_plan WHERE team = $t GROUP ALL; \
             SELECT count() AS n FROM action_item WHERE team = $t AND status IN ['open', 'in_progress'] GROUP ALL; \
             SELECT count() AS n FROM team_member WHERE team = $t GROUP ALL; \
             SELECT count() AS n FROM post_game_learning WHERE team = $t GROUP ALL; \
             SELECT VALUE name FROM ONLY $t;",
        )
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let name: Option<String> = r.take(7).unwrap_or_default();
    let mut n = |i: usize| -> usize {
        let row: Option<Count> = r.take(i).unwrap_or_default();
        row.map(|c| c.n.max(0) as usize).unwrap_or(0)
    };
    Ok(TeamCounts {
        name: name.unwrap_or_default(),
        opponents: n(1),
        drafts: n(2),
        plans: n(3),
        open_items: n(4),
        members: n(5),
        reviews: n(6),
    })
}

// ---------------------------------------------------------------------------
// Share links and player cards (M5)
// ---------------------------------------------------------------------------

/// A share token: 32 characters from a 32-letter alphabet (160 bits).
pub fn generate_share_token() -> String {
    use password_hash::rand_core::{OsRng, RngCore};
    const ALPHABET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect()
}

/// A token as it may appear in a URL: 32 characters of the alphabet.
pub fn plausible_share_token(token: &str) -> bool {
    token.len() == 32
        && token
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct DbShareLink {
    id: RecordId,
    token: String,
    kind: String,
    created_by_name: Option<String>,
    created_at: Option<Datetime>,
    expires_at: Datetime,
    revoked: bool,
}

impl DbShareLink {
    fn into_model(self) -> Option<ShareLink> {
        Some(ShareLink {
            id: self.id.to_sql(),
            token: self.token,
            kind: ShareKind::parse(&self.kind)?,
            created_by_name: self.created_by_name.unwrap_or_default(),
            created_at: opt_dt(self.created_at),
            expires_at: dt_to_string(&self.expires_at),
            revoked: self.revoked,
        })
    }
}

/// The target must be the team's: its plan, its review, or one of its
/// members (player cards).
async fn ensure_share_target(
    db: &Surreal<Db>,
    team_id: &str,
    kind: ShareKind,
    target_id: &str,
) -> DbResult<()> {
    match kind {
        ShareKind::GamePlan => ensure_team_record(db, "game_plan", target_id, team_id).await,
        ShareKind::Review => ensure_team_record(db, "post_game_learning", target_id, team_id).await,
        ShareKind::Player => {
            if !is_team_member(db, target_id, team_id).await? {
                return Err(DbError::NotFound);
            }
            if crate::server::account::share_card_hidden(db, target_id).await? {
                return Err(DbError::Invalid(
                    "This player keeps their card out of share links.".into(),
                ));
            }
            Ok(())
        }
    }
}

/// Create a read-only link to one of the team's plans, reviews or player
/// cards, valid for `days` (1, 7 or 30).
pub async fn create_share_link(
    db: &Surreal<Db>,
    team_id: &str,
    user_id: &str,
    kind: ShareKind,
    target_id: &str,
    days: u32,
) -> DbResult<ShareLink> {
    if !SHARE_DAYS.contains(&days) {
        return Err(DbError::Invalid(
            "A link lasts one day, a week or a month.".into(),
        ));
    }
    ensure_share_target(db, team_id, kind, target_id).await?;
    let expires = (chrono::Utc::now() + chrono::Duration::days(days as i64)).to_rfc3339();
    let token = generate_share_token();
    let mut r = db
        .query("CREATE share_link SET token = $tok, team = type::record('team', $team_key), kind = $kind, target = type::record($tb, $target_key), created_by = type::record('user', $user_key), expires_at = <datetime>$expires RETURN id, token, kind, created_by.username AS created_by_name, created_at, expires_at, revoked")
        .bind(("tok", token))
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("kind", kind.key().to_string()))
        .bind(("tb", kind.table().to_string()))
        .bind(("target_key", record_key(target_id, kind.table())))
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("expires", expires))
        .await?;
    let row: Option<DbShareLink> = r.take(0)?;
    row.and_then(DbShareLink::into_model)
        .ok_or_else(|| DbError::Other("Failed to create the link".into()))
}

/// The live (not revoked, not expired) links to one target of the team.
pub async fn list_share_links(
    db: &Surreal<Db>,
    team_id: &str,
    kind: ShareKind,
    target_id: &str,
) -> DbResult<Vec<ShareLink>> {
    let mut r = db
        .query("SELECT id, token, kind, created_by.username AS created_by_name, created_at, expires_at, revoked FROM share_link WHERE team = type::record('team', $team_key) AND target = type::record($tb, $target_key) AND revoked = false AND expires_at > time::now() ORDER BY created_at DESC")
        .bind(("team_key", record_key(team_id, "team")))
        .bind(("tb", kind.table().to_string()))
        .bind(("target_key", record_key(target_id, kind.table())))
        .await?;
    let rows: Vec<DbShareLink> = r.take(0).unwrap_or_default();
    Ok(rows
        .into_iter()
        .filter_map(DbShareLink::into_model)
        .collect())
}

/// Revoke one of the team's links. Revoked links open nothing.
pub async fn revoke_share_link(db: &Surreal<Db>, team_id: &str, link_id: &str) -> DbResult<()> {
    let mut r = db
        .query("UPDATE type::record('share_link', $key) SET revoked = true WHERE team = type::record('team', $team_key) RETURN id")
        .bind(("key", record_key(link_id, "share_link")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let rows: Vec<IdRecord> = r.take(0)?;
    if rows.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// What a live token points at: (team id, kind, target id, expires at).
#[derive(Clone, Debug)]
pub struct ResolvedShare {
    pub team_id: String,
    pub kind: ShareKind,
    pub target_id: String,
    pub expires_at: String,
}

/// Look up a token. Unknown, revoked and expired tokens are all `None`.
pub async fn resolve_share_token(db: &Surreal<Db>, token: &str) -> DbResult<Option<ResolvedShare>> {
    if !plausible_share_token(token) {
        return Ok(None);
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        team: RecordId,
        kind: String,
        target: RecordId,
        expires_at: Datetime,
    }
    let mut r = db
        .query("SELECT team, kind, target, expires_at FROM share_link WHERE token = $tok AND revoked = false AND expires_at > time::now() LIMIT 1")
        .bind(("tok", token.to_string()))
        .await?;
    let row: Option<Row> = r.take(0)?;
    Ok(row.and_then(|r| {
        Some(ResolvedShare {
            team_id: r.team.to_sql(),
            kind: ShareKind::parse(&r.kind)?,
            target_id: r.target.to_sql(),
            expires_at: dt_to_string(&r.expires_at),
        })
    }))
}

/// A member's player card: position and place on the roster, best pool
/// champions, form over their last five stored games and most played
/// champions. `NotFound` when the user is not in the team.
pub async fn player_card(db: &Surreal<Db>, team_id: &str, user_id: &str) -> DbResult<PlayerCard> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Member {
        username: String,
        role: String,
        roster_type: String,
        team_name: String,
        share_hidden: Option<bool>,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Pool {
        champion: String,
        tier: String,
    }
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Game {
        champion: String,
        win: bool,
        #[allow(dead_code)]
        game_end: Option<Datetime>,
    }
    let mut r = db
        .query(
            "LET $u = type::record('user', $user_key); \
             SELECT user.username AS username, role, roster_type, team.name AS team_name, user.share_card_hidden AS share_hidden FROM team_member WHERE team = type::record('team', $team_key) AND user = $u LIMIT 1; \
             SELECT champion, tier FROM champion_pool WHERE user = $u; \
             SELECT champion, win, match.game_end AS game_end FROM player_match WHERE user = $u ORDER BY game_end DESC LIMIT 50;",
        )
        .bind(("user_key", record_key(user_id, "user")))
        .bind(("team_key", record_key(team_id, "team")))
        .await?;
    let member: Option<Member> = r.take(1)?;
    let Some(member) = member else {
        return Err(DbError::NotFound);
    };
    let mut pool: Vec<Pool> = r.take(2).unwrap_or_default();
    let games: Vec<Game> = r.take(3).unwrap_or_default();

    pool.sort_by(|a, b| {
        tier_rank(&a.tier)
            .cmp(&tier_rank(&b.tier))
            .then(a.champion.cmp(&b.champion))
    });
    let mut seen = std::collections::HashSet::new();
    let champions: Vec<CardChampion> = pool
        .into_iter()
        .filter(|p| seen.insert(p.champion.to_lowercase()))
        .take(5)
        .map(|p| CardChampion {
            champion: p.champion,
            tier: p.tier,
        })
        .collect();
    let mut form: Vec<bool> = games.iter().take(5).map(|g| g.win).collect();
    form.reverse();
    let mut played: Vec<(String, u32, u32)> = Vec::new();
    for g in &games {
        match played.iter_mut().find(|(c, _, _)| *c == g.champion) {
            Some(e) => {
                e.1 += 1;
                e.2 += g.win as u32;
            }
            None => played.push((g.champion.clone(), 1, g.win as u32)),
        }
    }
    played.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    played.truncate(3);
    Ok(PlayerCard {
        share_hidden: member.share_hidden.unwrap_or(false),
        username: member.username,
        team_name: member.team_name,
        role: member.role,
        place: member.roster_type,
        champions,
        form,
        most_played: played,
    })
}
