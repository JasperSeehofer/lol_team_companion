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
];

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
    let known: BTreeSet<i64> = migrations.iter().map(|m| m.version).collect();
    if let Some(unknown) = applied.iter().find(|v| !known.contains(v)) {
        return Err(DbError::Other(format!(
            "the database records migration {unknown}, which this build does not know; \
             it was migrated by a newer version of the app"
        )));
    }

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
/// accounts created before M1 normalised emails) are left unchanged and
/// logged: the unique index would reject the update, and picking which
/// account keeps the address is an owner decision. The lowercase row is the
/// one that can sign in.
async fn lowercase_emails(db: &Surreal<Db>) -> DbResult<()> {
    let mut r = db.query("SELECT id, email FROM user").await?;
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
        if group.len() > 1 {
            let ids: Vec<String> = group.iter().map(|r| r.id.to_sql()).collect();
            tracing::warn!(
                users = ?ids,
                "emails differ only by case; left unchanged (resolve by hand)"
            );
            continue;
        }
        let row = needs_change[0];
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
