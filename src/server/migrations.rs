//! Versioned schema migrations.
//!
//! `schema.surql` is the baseline: it is applied on every start and only uses
//! idempotent statements (`DEFINE ... IF NOT EXISTS`), so a fresh database gets
//! every table, field and plain index from it. Anything the baseline cannot do
//! safely on an existing database lives here: data backfills, type changes and
//! unique constraints that need clean data first.
//!
//! [`run`] applies every migration in [`MIGRATIONS`] whose version is not yet
//! recorded in the `schema_migration` table, in version order, once each:
//!
//! - A SurrealQL migration ([`Step::Surql`]) runs in one transaction together
//!   with the insert of its `schema_migration` row, so it is applied completely
//!   or not at all.
//! - A Rust migration ([`Step::Rust`]) is for work that needs per-row logic
//!   (parsing legacy values, resolving duplicates). It can span several
//!   queries and therefore cannot be one transaction; it must be idempotent so
//!   a run interrupted half-way is safe to repeat. Its row is written after it
//!   succeeds.
//!
//! Migrations also run on a fresh database (right after the baseline), so each
//! one must be a no-op or harmless there.
//!
//! A failed migration stops the server at startup (see `db::init_db`). A
//! database that records a version this build does not know (it was migrated
//! by a newer build) is refused as well.
//!
//! How to add one: see "How to add a migration" in docs/ARCHITECTURE.md.

use std::collections::{BTreeSet, HashMap};
use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;
use surrealdb::{
    engine::local::Db,
    types::{RecordId, SurrealValue, ToSql},
    Surreal,
};

use crate::server::db::{record_key, DbError, DbResult};

type RustStep =
    for<'a> fn(&'a Surreal<Db>) -> Pin<Box<dyn Future<Output = DbResult<()>> + Send + 'a>>;

pub enum Step {
    /// SurrealQL, run inside one transaction with the bookkeeping insert.
    Surql(&'static str),
    /// Rust code for per-row work. Must be idempotent.
    Rust(RustStep),
}

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub step: Step,
}

/// Every migration, in version order. Never edit or renumber one that has
/// shipped; add a new one instead.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "lowercase_emails",
        step: Step::Rust(|db| Box::pin(lowercase_emails(db))),
    },
    Migration {
        version: 2,
        name: "username_lc_unique",
        step: Step::Rust(|db| Box::pin(username_lc_unique(db))),
    },
    Migration {
        version: 3,
        name: "is_admin_not_null",
        step: Step::Surql(
            "UPDATE user SET is_admin = false WHERE is_admin = NONE; \
             DEFINE FIELD OVERWRITE is_admin ON user TYPE bool DEFAULT false;",
        ),
    },
    Migration {
        version: 4,
        name: "drop_legacy_sessions",
        // Session rows written before M1 have no `expires_at`. Their auth hash
        // format is also obsolete, so they could never be used again.
        step: Step::Surql("DELETE sessions WHERE expires_at = NONE;"),
    },
    Migration {
        version: 5,
        name: "string_ids_to_record_links",
        step: Step::Rust(|db| Box::pin(string_ids_to_record_links(db))),
    },
    Migration {
        version: 6,
        name: "one_team_per_user_unique",
        step: Step::Rust(|db| Box::pin(one_team_per_user_unique(db))),
    },
    Migration {
        version: 7,
        name: "drop_user_theme",
        // M2 left one theme; the per-user theme field is dead data.
        step: Step::Surql("REMOVE FIELD IF EXISTS theme ON user; UPDATE user UNSET theme;"),
    },
    Migration {
        version: 8,
        name: "roster_places",
        step: Step::Surql(ROSTER_PLACES),
    },
    Migration {
        version: 9,
        name: "normalise_pool_roles",
        step: Step::Rust(|db| Box::pin(normalise_pool_roles(db))),
    },
    Migration {
        version: 10,
        name: "bug_report_statuses_v2",
        // M7: statuses users can follow. `closed` had no recorded meaning
        // (fixed or declined), so those rows go back to `triaged` with a note
        // for an admin instead of an invented decision. `status_changed_at`
        // stays unset: the move is bookkeeping, not news for the reporter.
        step: Step::Surql(BUG_REPORT_STATUSES_V2),
    },
    Migration {
        version: 11,
        name: "community_seed_content",
        // M7: the first what-next proposals (unpublished), the proposed
        // surveys (drafts, unapproved) and the money texts (unapproved).
        // Idempotent: rows are keyed by seed_key / slug.
        step: Step::Rust(|db| Box::pin(community_seed_content(db))),
    },
    Migration {
        version: 12,
        name: "drop_user_mode",
        // The Solo/Team workspace switch went in M5; nothing has read
        // `user.mode` since (owner decision: drop it).
        step: Step::Surql("REMOVE FIELD IF EXISTS mode ON user; UPDATE user UNSET mode;"),
    },
];

const BUG_REPORT_STATUSES_V2: &str = "\
    DEFINE FIELD OVERWRITE status ON bug_report TYPE string DEFAULT 'new'; \
    UPDATE bug_report SET status = 'new' WHERE status = 'open'; \
    UPDATE bug_report SET status = 'triaged', \
        admin_note = 'Closed before statuses existed; set a final status.' \
      WHERE status = 'closed'; \
    UPDATE bug_report SET status = 'new' \
      WHERE status NOT IN ['new', 'triaged', 'planned', 'fixed', 'wont_fix']; \
    DEFINE FIELD OVERWRITE status ON bug_report TYPE string DEFAULT 'new' \
      ASSERT $value IN ['new', 'triaged', 'planned', 'fixed', 'wont_fix'];";

/// M7 seed content (see `server::community::seeds`). Creates a row only when
/// none with its key exists, so an admin's edits are never overwritten.
pub async fn community_seed_content(db: &Surreal<Db>) -> DbResult<()> {
    use crate::server::community::{seeds, wording_hash};
    db.query(
        "FOR $p IN $proposals { \
             IF array::len(SELECT VALUE id FROM proposal WHERE seed_key = $p.seed_key) = 0 { \
                 CREATE proposal SET seed_key = $p.seed_key, title = $p.title, summary = $p.summary, \
                     status = $p.status, source = $p.source, shipped_in = $p.shipped_in, \
                     published = false, created_at = time::now(), status_changed_at = time::now(); \
             }; \
         }; \
         FOR $s IN $surveys { \
             IF array::len(SELECT VALUE id FROM survey WHERE seed_key = $s.seed_key) = 0 { \
                 CREATE survey SET seed_key = $s.seed_key, question = $s.question, kind = $s.kind, \
                     choices = $s.choices, scale_low_label = $s.scale_low_label, \
                     scale_high_label = $s.scale_high_label, trigger = $s.trigger, status = 'draft', \
                     wording_hash = $s.wording_hash, note = $s.note, created_at = time::now(); \
             }; \
         }; \
         FOR $t IN $texts { \
             IF array::len(SELECT VALUE id FROM site_text WHERE slug = $t.slug) = 0 { \
                 CREATE site_text SET slug = $t.slug, body = $t.body, approved = false, \
                     updated_at = time::now(); \
             }; \
         };",
    )
    .bind(("proposals", seeds::proposals()))
    .bind(("surveys", seeds::surveys(wording_hash)))
    .bind(("texts", seeds::texts()))
    .await?
    .check()?;
    Ok(())
}

/// M5: one spelling per position, a real place for every member (starting
/// five, substitutes, coaches, or not placed yet), the per-page view
/// preferences of views that no longer exist dropped, and links to deleted
/// drafts replaced by a "draft deleted" mark.
///
/// Before: `role` held positions in mixed spellings plus "coach", "sub" and
/// "unassigned"; `roster_type` was "starter" or "sub" (everyone not starting,
/// shown as the bench). Now a non-starter with a position becomes a
/// substitute (the old bench), one without a position is "member" (not
/// placed yet), and coaches get their own place.
const ROSTER_PLACES: &str = "\
    UPDATE team_member SET role = string::lowercase(string::trim(role)); \
    UPDATE team_member SET role = 'jungle' WHERE role IN ['jgl', 'jg', 'jungler']; \
    UPDATE team_member SET role = 'mid' WHERE role IN ['middle']; \
    UPDATE team_member SET role = 'bot' WHERE role IN ['bottom', 'adc', 'carry', 'marksman']; \
    UPDATE team_member SET role = 'support' WHERE role IN ['sup', 'supp', 'utility']; \
    UPDATE team_member SET roster_type = 'coach', role = 'unassigned' WHERE role = 'coach'; \
    UPDATE team_member SET role = 'unassigned' \
        WHERE role NOT IN ['top', 'jungle', 'mid', 'bot', 'support', 'unassigned']; \
    UPDATE team_member SET roster_type = 'member' WHERE roster_type = 'starter' AND role = 'unassigned'; \
    UPDATE team_member SET roster_type = 'substitute' \
        WHERE roster_type NOT IN ['starter', 'coach'] AND role != 'unassigned'; \
    UPDATE team_member SET roster_type = 'member' \
        WHERE roster_type NOT IN ['starter', 'coach', 'substitute']; \
    DEFINE FIELD OVERWRITE role ON team_member TYPE string DEFAULT 'unassigned' \
        ASSERT $value IN ['top', 'jungle', 'mid', 'bot', 'support', 'unassigned']; \
    DEFINE FIELD OVERWRITE roster_type ON team_member TYPE string DEFAULT 'member' \
        ASSERT $value IN ['starter', 'substitute', 'coach', 'member']; \
    REMOVE FIELD IF EXISTS draft_mode ON user; \
    REMOVE FIELD IF EXISTS team_dashboard_mode ON user; \
    REMOVE FIELD IF EXISTS solo_mode ON user; \
    UPDATE user UNSET draft_mode, team_dashboard_mode, solo_mode; \
    UPDATE game_plan SET draft = NONE, draft_deleted = true \
        WHERE draft != NONE AND !record::exists(draft); \
    UPDATE post_game_learning SET draft_id = NONE, draft_deleted = true \
        WHERE draft_id != NONE AND !record::exists(draft_id);";

#[derive(Debug, Deserialize, SurrealValue)]
struct PoolRoleRow {
    id: RecordId,
    user: RecordId,
    champion: String,
    role: String,
    #[allow(dead_code)]
    created_at: Option<surrealdb::types::Datetime>,
}

/// M5: champion pool and champion note roles in one spelling (`Role::key`:
/// top, jungle, mid, bot, support). Pool rows that become duplicates (the
/// same champion stored once as "ADC" and once as "bot") keep the oldest row.
/// Roles no spelling maps are left as they are. Idempotent: rows already in
/// the stored spelling are skipped.
async fn normalise_pool_roles(db: &Surreal<Db>) -> DbResult<()> {
    use crate::components::ui::Role;

    let mut r = db
        .query("SELECT id, user, champion, role, created_at FROM champion_pool ORDER BY created_at ASC, id ASC")
        .await?;
    let rows: Vec<PoolRoleRow> = r.take(0)?;
    let mut seen: BTreeSet<(String, String, String)> = BTreeSet::new();
    let (mut renamed, mut removed) = (0usize, 0usize);
    for row in &rows {
        let Some(role) = Role::parse(&row.role) else {
            continue;
        };
        let key = (
            row.user.to_sql(),
            row.champion.to_lowercase(),
            role.key().to_string(),
        );
        let id_key = record_key(&row.id.to_sql(), "champion_pool");
        if !seen.insert(key) {
            db.query("DELETE type::record('champion_pool', $key)")
                .bind(("key", id_key))
                .await?
                .check()?;
            removed += 1;
        } else if row.role != role.key() {
            db.query("UPDATE type::record('champion_pool', $key) SET role = $role")
                .bind(("key", id_key))
                .bind(("role", role.key().to_string()))
                .await?
                .check()?;
            renamed += 1;
        }
    }

    #[derive(Debug, Deserialize, SurrealValue)]
    struct NoteRole {
        id: RecordId,
        role: String,
    }
    let mut r = db.query("SELECT id, role FROM champion_note").await?;
    let notes: Vec<NoteRole> = r.take(0)?;
    for n in notes {
        if let Some(role) = Role::parse(&n.role) {
            if n.role != role.key() {
                db.query("UPDATE type::record('champion_note', $key) SET role = $role")
                    .bind(("key", record_key(&n.id.to_sql(), "champion_note")))
                    .bind(("role", role.key().to_string()))
                    .await?
                    .check()?;
                renamed += 1;
            }
        }
    }
    if renamed > 0 || removed > 0 {
        tracing::info!(renamed, removed, "pool and note roles normalised");
    }
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct AppliedRow {
    version: i64,
}

/// Versions recorded in `schema_migration`.
pub async fn applied_versions(db: &Surreal<Db>) -> DbResult<BTreeSet<i64>> {
    let mut r = db.query("SELECT version FROM schema_migration").await?;
    let rows: Vec<AppliedRow> = r.take(0)?;
    Ok(rows.into_iter().map(|r| r.version).collect())
}

/// Refuse a database that records a migration `migrations` does not know:
/// it was migrated by a newer build.
fn refuse_unknown(applied: &BTreeSet<i64>, migrations: &[Migration]) -> DbResult<()> {
    let known: BTreeSet<i64> = migrations.iter().map(|m| m.version).collect();
    if let Some(unknown) = applied.iter().find(|v| !known.contains(v)) {
        return Err(DbError::Other(format!(
            "the database records migration {unknown}, which this build does not know; \
             it was migrated by a newer version of the app"
        )));
    }
    Ok(())
}

/// Check, before anything is written, that this build knows every migration
/// the database records. `db::prepare` calls it before it applies the
/// `schema.surql` baseline, so an older image started on a newer database
/// stops without writing its older definitions into it. On a fresh database
/// (no `schema_migration` table yet) there is nothing to check.
pub async fn check_known(db: &Surreal<Db>) -> DbResult<()> {
    let applied = match applied_versions(db).await {
        Ok(applied) => applied,
        // SurrealDB 3 reports a table that was never defined as not found.
        Err(DbError::Surreal(e))
            if matches!(e.details(), surrealdb::types::ErrorDetails::NotFound(_)) =>
        {
            return Ok(());
        }
        Err(e) => return Err(e),
    };
    refuse_unknown(&applied, MIGRATIONS)
}

/// Apply every pending migration in order. Returns the versions applied by
/// this call.
pub async fn run(db: &Surreal<Db>) -> DbResult<Vec<i64>> {
    run_list(db, MIGRATIONS).await
}

/// [`run`] over an explicit list (tests use their own lists).
pub async fn run_list(db: &Surreal<Db>, migrations: &[Migration]) -> DbResult<Vec<i64>> {
    // The list must be strictly increasing; a mistake here is a build bug.
    if migrations.windows(2).any(|w| w[0].version >= w[1].version) {
        return Err(DbError::Other(
            "migrations are not in strictly increasing version order".into(),
        ));
    }

    let applied = applied_versions(db).await?;
    refuse_unknown(&applied, migrations)?;

    let mut done = Vec::new();
    for m in migrations.iter().filter(|m| !applied.contains(&m.version)) {
        tracing::info!(version = m.version, name = m.name, "applying migration");
        apply_one(db, m).await.map_err(|e| {
            DbError::Other(format!("migration {} ({}) failed: {e}", m.version, m.name))
        })?;
        done.push(m.version);
    }
    if done.is_empty() {
        tracing::debug!("database schema is up to date");
    }
    Ok(done)
}

async fn apply_one(db: &Surreal<Db>, m: &Migration) -> DbResult<()> {
    const RECORD: &str =
        "CREATE schema_migration SET version = $mig_version, name = $mig_name, applied_at = time::now();";
    match &m.step {
        Step::Surql(body) => {
            // Body text is a compile-time constant from MIGRATIONS.
            let query = format!("BEGIN TRANSACTION; {body} {RECORD} COMMIT TRANSACTION;");
            let response = db
                .query(query)
                .bind(("mig_version", m.version))
                .bind(("mig_name", m.name))
                .await?;
            crate::server::db::check_tx(response)?;
        }
        Step::Rust(f) => {
            f(db).await?;
            db.query(RECORD)
                .bind(("mig_version", m.version))
                .bind(("mig_name", m.name))
                .await?
                .check()?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Migration bodies
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, SurrealValue)]
struct UserEmailRow {
    id: RecordId,
    email: String,
}

/// Store every email lowercase so logins can use the unique `email` index
/// (`email = $email`) instead of scanning with `string::lowercase`.
///
/// Rows whose lowercase form collides with another row (possible only for
/// accounts created before M1 normalised emails) are logged for the owner to
/// resolve. Login matches the lowercase form exactly, so only a lowercase row
/// can sign in: when the group already has one, the others are left
/// unchanged; when none is lowercase, the oldest account is lowercased so
/// that one of them can still sign in (the others are left unchanged).
async fn lowercase_emails(db: &Surreal<Db>) -> DbResult<()> {
    let mut r = db
        .query("SELECT id, email, created_at FROM user ORDER BY created_at ASC, id ASC")
        .await?;
    let rows: Vec<UserEmailRow> = r.take(0)?;

    let mut by_lower: HashMap<String, Vec<&UserEmailRow>> = HashMap::new();
    for row in &rows {
        by_lower
            .entry(row.email.trim().to_lowercase())
            .or_default()
            .push(row);
    }

    for (lower, group) in by_lower {
        let needs_change: Vec<&&UserEmailRow> = group.iter().filter(|r| r.email != lower).collect();
        if needs_change.is_empty() {
            continue;
        }
        let row = if group.len() > 1 {
            let ids: Vec<String> = group.iter().map(|r| r.id.to_sql()).collect();
            if needs_change.len() < group.len() {
                tracing::warn!(
                    users = ?ids,
                    "emails differ only by case; left unchanged (resolve by hand)"
                );
                continue;
            }
            // No lowercase row: nobody in the group could sign in. Keep the
            // oldest account usable (rows are in creation order).
            tracing::warn!(
                users = ?ids,
                lowercased = %group[0].id.to_sql(),
                "emails differ only by case; lowercased the oldest, left the rest (resolve by hand)"
            );
            group[0]
        } else {
            *needs_change[0]
        };
        db.query("UPDATE type::record('user', $key) SET email = $email")
            .bind(("key", record_key(&row.id.to_sql(), "user")))
            .bind(("email", lower.clone()))
            .await?
            .check()?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct UserNameRow {
    id: RecordId,
    username: String,
    username_lc: Option<String>,
}

/// Backfill `user.username_lc` (trimmed, lowercase username) and add the
/// unique index on it.
///
/// Usernames that collide case-insensitively (only possible for accounts
/// created before M1) get a numeric suffix on every account except the oldest
/// (`Name` stays, `name` becomes `name-2`); each rename is logged.
async fn username_lc_unique(db: &Surreal<Db>) -> DbResult<()> {
    let mut r = db
        .query("SELECT id, username, username_lc, created_at FROM user ORDER BY created_at ASC, id ASC")
        .await?;
    let rows: Vec<UserNameRow> = r.take(0)?;

    let mut taken: BTreeSet<String> = BTreeSet::new();
    let mut updates: Vec<(String, String, String)> = Vec::new(); // (key, username, username_lc)
    for row in &rows {
        let base = row.username.trim().to_string();
        let mut name = base.clone();
        let mut n = 2;
        while taken.contains(&name.to_lowercase()) {
            name = format!("{base}-{n}");
            n += 1;
        }
        let lc = name.to_lowercase();
        taken.insert(lc.clone());
        if name != row.username {
            tracing::warn!(user = %row.id.to_sql(), from = %row.username, to = %name,
                "renamed a username that collided case-insensitively");
        }
        if name != row.username || row.username_lc.as_deref() != Some(lc.as_str()) {
            updates.push((record_key(&row.id.to_sql(), "user"), name, lc));
        }
    }

    for (key, username, lc) in updates {
        db.query("UPDATE type::record('user', $key) SET username = $username, username_lc = $lc")
            .bind(("key", key))
            .bind(("username", username))
            .bind(("lc", lc))
            .await?
            .check()?;
    }

    db.query(
        "DEFINE FIELD OVERWRITE username_lc ON user TYPE string; \
         DEFINE INDEX IF NOT EXISTS username_lc_unique ON user FIELDS username_lc UNIQUE;",
    )
    .await?
    .check()?;
    Ok(())
}

/// Fields that stored a record ID as a string before M6, and the table the ID
/// points into.
pub const LINK_FIELDS: &[(&str, &str, &str)] = &[
    ("draft", "series_id", "series"),
    ("game_plan", "draft", "draft"),
    ("post_game_learning", "game_plan_id", "game_plan"),
    ("post_game_learning", "draft_id", "draft"),
    ("series", "opponent_id", "opponent"),
    ("action_item", "source_review", "post_game_learning"),
    ("checklist_instance", "game_plan_id", "game_plan"),
    ("checklist_instance", "template_id", "checklist_template"),
];

#[derive(Debug, Deserialize, SurrealValue)]
struct StringLinkRow {
    id: RecordId,
    v: String,
}

/// The key of `value` when it is a `target:key` ID string; `None` for empty
/// strings and anything that points elsewhere.
pub fn link_key(value: &str, target: &str) -> Option<String> {
    let value = value.trim();
    let key = value.strip_prefix(target)?.strip_prefix(':')?;
    let key = key
        .strip_prefix('⟨')
        .and_then(|k| k.strip_suffix('⟩'))
        .unwrap_or(key);
    if key.is_empty() {
        None
    } else {
        Some(key.to_string())
    }
}

/// Convert the string IDs in [`LINK_FIELDS`] into record links.
///
/// Per field: widen the type to accept both forms, rewrite every string value
/// (`"draft:abc"` becomes `draft:abc`; empty or malformed values become NONE
/// and are logged), then tighten the type to `option<record<target>>`. Links
/// to records that no longer exist are kept as they are (SurrealDB allows
/// dangling links; reads already treat a missing target as absent).
/// Idempotent: a second run finds no string values.
async fn string_ids_to_record_links(db: &Surreal<Db>) -> DbResult<()> {
    for (table, field, target) in LINK_FIELDS {
        // Table, field and target names are compile-time constants.
        db.query(format!(
            "DEFINE FIELD OVERWRITE {field} ON {table} TYPE option<string | record<{target}>>;"
        ))
        .await?
        .check()?;

        let mut r = db
            .query(format!(
                "SELECT id, {field} AS v FROM {table} WHERE type::is_string({field})"
            ))
            .await?;
        let rows: Vec<StringLinkRow> = r.take(0)?;
        let mut converted = 0usize;
        for row in rows {
            let row_key = record_key(&row.id.to_sql(), table);
            let key = link_key(&row.v, target);
            if key.is_none() && !row.v.trim().is_empty() {
                tracing::warn!(record = %row.id.to_sql(), field, value = %row.v,
                    "unparseable link ID cleared");
            }
            db.query(format!(
                "UPDATE type::record($tb, $row_key) SET {field} = \
                 IF $key = NONE {{ NONE }} ELSE {{ type::record($target, $key) }}"
            ))
            .bind(("tb", *table))
            .bind(("row_key", row_key))
            .bind(("target", *target))
            .bind(("key", key))
            .await?
            .check()?;
            converted += 1;
        }
        if converted > 0 {
            tracing::info!(
                table,
                field,
                converted,
                "string IDs converted to record links"
            );
        }

        db.query(format!(
            "DEFINE FIELD OVERWRITE {field} ON {table} TYPE option<record<{target}>>;"
        ))
        .await?
        .check()?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, SurrealValue)]
struct MembershipRow {
    id: RecordId,
    user: RecordId,
    team: RecordId,
    leads: bool,
}

/// Enforce "a user belongs to at most one team" in the database with a
/// unique index on `team_member.user` (until now only the application
/// checked it, under a process lock).
///
/// Users created before M1 can have several memberships. For each such user
/// one membership is kept: the team they lead if there is one, otherwise the
/// membership with the lowest record ID (the one `get_user_team_id`, which
/// orders by ID, already resolved to, so such a user's view does not
/// change; a leader may now see the team they lead instead). Record IDs are
/// random, so this is not necessarily the oldest. The others are deleted
/// and logged.
async fn one_team_per_user_unique(db: &Surreal<Db>) -> DbResult<()> {
    let mut r = db
        .query(
            "SELECT id, user, team, (team.created_by = user) AS leads FROM team_member ORDER BY id ASC",
        )
        .await?;
    let rows: Vec<MembershipRow> = r.take(0)?;

    let mut by_user: std::collections::BTreeMap<String, Vec<&MembershipRow>> =
        std::collections::BTreeMap::new();
    for row in &rows {
        by_user.entry(row.user.to_sql()).or_default().push(row);
    }
    for (user, memberships) in by_user {
        if memberships.len() < 2 {
            continue;
        }
        let keep = memberships
            .iter()
            .find(|m| m.leads)
            .unwrap_or(&memberships[0])
            .id
            .to_sql();
        for m in memberships.iter().filter(|m| m.id.to_sql() != keep) {
            tracing::warn!(%user, team = %m.team.to_sql(), kept = %keep,
                "removed an extra team membership (one team per user)");
            db.query("DELETE type::record('team_member', $key)")
                .bind(("key", record_key(&m.id.to_sql(), "team_member")))
                .await?
                .check()?;
        }
    }

    db.query(
        "REMOVE INDEX IF EXISTS team_member_user ON team_member; \
         DEFINE INDEX IF NOT EXISTS team_member_user_unique ON team_member FIELDS user UNIQUE;",
    )
    .await?
    .check()?;
    Ok(())
}
