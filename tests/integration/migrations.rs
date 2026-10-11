//! M6 migration runner: fresh databases, legacy data, idempotence, failure
//! handling and the indexes the baseline defines.

use lol_team_companion::server::db::{self, DbError};
use lol_team_companion::server::migrations::{self, Migration, Step, MIGRATIONS};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::types::{SurrealValue, Value};
use surrealdb::Surreal;

/// An in-memory DB with only the baseline schema applied (no migrations), as
/// a database created before M6 would look after the first M6 start applies
/// `schema.surql`.
async fn baseline_only() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    db.query(include_str!("../../schema.surql"))
        .await
        .unwrap()
        .check()
        .unwrap();
    db
}

async fn info_for_table(db: &Surreal<Db>, table: &str) -> String {
    let mut r = db.query(format!("INFO FOR TABLE {table}")).await.unwrap();
    let v: Value = r.take(0).unwrap();
    format!("{v:?}")
}

#[tokio::test]
async fn fresh_database_records_every_migration_once() {
    let db = crate::common::test_db().await;
    let applied = migrations::applied_versions(&db).await.unwrap();
    let all: Vec<i64> = MIGRATIONS.iter().map(|m| m.version).collect();
    assert_eq!(applied.into_iter().collect::<Vec<_>>(), all);

    // A second run is a no-op.
    let again = migrations::run(&db).await.unwrap();
    assert!(again.is_empty(), "nothing pending: {again:?}");
    db::prepare(&db).await.unwrap();
}

#[tokio::test]
async fn scope_fields_are_indexed() {
    let db = crate::common::test_db().await;
    let expected: &[(&str, &[&str])] = &[
        (
            "user",
            &["email_unique", "username_lc_unique", "user_riot_puuid"],
        ),
        ("team_member", &["team_member_team", "team_member_user"]),
        ("draft", &["draft_team"]),
        ("draft_action", &["draft_action_draft"]),
        ("game_plan", &["game_plan_team"]),
        ("post_game_learning", &["post_game_learning_team"]),
        ("draft_tree", &["draft_tree_team"]),
        ("draft_tree_node", &["draft_tree_node_tree"]),
        ("action_item", &["action_item_team"]),
        ("opponent", &["opponent_team"]),
        ("opponent_player", &["opponent_player_opponent"]),
        ("team_note", &["team_note_team"]),
        ("series", &["series_team"]),
        ("player_match", &["player_match_user", "player_match_match"]),
        ("match", &["match_id_unique", "match_team"]),
        ("sessions", &["sessions_expires_at"]),
        ("invite", &["invite_code_unique"]),
    ];
    for (table, indexes) in expected {
        let info = info_for_table(&db, table).await;
        for idx in *indexes {
            assert!(info.contains(idx), "{table} is missing index {idx}: {info}");
        }
    }
    let user = info_for_table(&db, "user").await;
    assert!(
        user.contains("DEFINE INDEX username_lc_unique ON user FIELDS username_lc UNIQUE"),
        "{user}"
    );
    assert!(
        user.contains("DEFINE FIELD is_admin ON user TYPE bool DEFAULT false"),
        "{user}"
    );
}

#[tokio::test]
async fn legacy_rows_are_backfilled() {
    let db = baseline_only().await;
    // Rows as they could exist before M6: mixed-case emails, no username_lc,
    // is_admin NONE, case-colliding usernames, a pre-M1 session row.
    db.query(
        "CREATE user:old1 SET username = 'Bob', email = 'Bob@Example.com', password_hash = 'h', created_at = d'2024-01-01T00:00:00Z'; \
         CREATE user:old2 SET username = 'bob', email = 'other@example.com', password_hash = 'h', created_at = d'2024-02-01T00:00:00Z'; \
         CREATE user:old3 SET username = ' Carol ', email = 'carol@example.com', password_hash = 'h', is_admin = true; \
         CREATE user:dup1 SET username = 'Dee', email = 'dee@example.com', password_hash = 'h'; \
         CREATE user:dup2 SET username = 'DeeToo', email = 'DEE@example.com', password_hash = 'h'; \
         CREATE sessions:legacy SET data = 'x'; \
         CREATE sessions:current SET data = 'y', expires_at = 9999999999;",
    )
    .await
    .unwrap()
    .check()
    .unwrap();

    let applied = migrations::run(&db).await.unwrap();
    assert_eq!(applied.len(), MIGRATIONS.len());

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct U {
        username: String,
        username_lc: String,
        email: String,
        is_admin: bool,
    }
    let get = |key: &'static str| {
        let db = &db;
        async move {
            let mut r = db
                .query(
                    "SELECT username, username_lc, email, is_admin FROM type::record('user', $k)",
                )
                .bind(("k", key))
                .await
                .unwrap();
            let u: Option<U> = r.take(0).unwrap();
            u.unwrap()
        }
    };

    let old1 = get("old1").await;
    assert_eq!(old1.email, "bob@example.com");
    assert_eq!(old1.username, "Bob", "the oldest account keeps its name");
    assert_eq!(old1.username_lc, "bob");
    assert!(!old1.is_admin);

    let old2 = get("old2").await;
    assert_eq!(
        old2.username, "bob-2",
        "a later case-collision gets a suffix"
    );
    assert_eq!(old2.username_lc, "bob-2");

    let old3 = get("old3").await;
    assert_eq!(old3.username, "Carol", "usernames are trimmed");
    assert_eq!(old3.username_lc, "carol");
    assert!(old3.is_admin, "an existing true flag is kept");

    // Emails that differ only by case stay as they were (logged).
    assert_eq!(get("dup1").await.email, "dee@example.com");
    assert_eq!(get("dup2").await.email, "DEE@example.com");

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Id {
        id: surrealdb::types::RecordId,
    }
    let mut r = db.query("SELECT id FROM sessions").await.unwrap();
    let sessions: Vec<Id> = r.take(0).unwrap();
    assert_eq!(
        sessions.len(),
        1,
        "only the session with expires_at survives"
    );

    // The unique index now holds.
    let dup = db::create_user(&db, "BOB".into(), "new@example.com".into(), "h".into()).await;
    match dup {
        Err(DbError::Invalid(msg)) => assert!(msg.contains("username is taken"), "{msg}"),
        other => panic!("expected a taken username, got {other:?}"),
    }
    // And a login lookup by exact (lowercase) email finds the migrated row.
    let mut r = db
        .query("SELECT id FROM user WHERE email = $e")
        .bind(("e", "bob@example.com"))
        .await
        .unwrap();
    let found: Vec<Id> = r.take(0).unwrap();
    assert_eq!(found.len(), 1);
}

#[tokio::test]
async fn a_failed_surql_migration_rolls_back_and_is_not_recorded() {
    let db = crate::common::test_db().await;
    const BROKEN: &[Migration] = &[Migration {
        version: 1,
        name: "broken",
        step: Step::Surql("CREATE marker:one SET x = 1; THROW 'stop here';"),
    }];
    // Use a fresh namespace so the real migrations' rows do not interfere.
    db.use_ns("test").use_db("other").await.unwrap();
    db.query(include_str!("../../schema.surql"))
        .await
        .unwrap()
        .check()
        .unwrap();

    let err = migrations::run_list(&db, BROKEN).await.unwrap_err();
    assert!(err.to_string().contains("broken"), "{err}");
    assert!(migrations::applied_versions(&db).await.unwrap().is_empty());
    let mut r = db.query("SELECT * FROM marker").await.unwrap();
    let rows: Vec<Value> = r.take(0).unwrap_or_default();
    assert!(rows.is_empty(), "the transaction rolled back: {rows:?}");
}

#[tokio::test]
async fn a_database_from_a_newer_build_is_refused() {
    let db = crate::common::test_db().await;
    db.query(
        "CREATE schema_migration SET version = 9999, name = 'future', applied_at = time::now()",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    let err = migrations::run(&db).await.unwrap_err();
    assert!(err.to_string().contains("9999"), "{err}");
    assert!(db::prepare(&db).await.is_err());
}

#[tokio::test]
async fn migrations_must_be_in_order() {
    let db = crate::common::test_db().await;
    const OUT_OF_ORDER: &[Migration] = &[
        Migration {
            version: 2,
            name: "b",
            step: Step::Surql("RETURN 1;"),
        },
        Migration {
            version: 1,
            name: "a",
            step: Step::Surql("RETURN 1;"),
        },
    ];
    assert!(migrations::run_list(&db, OUT_OF_ORDER).await.is_err());
}
