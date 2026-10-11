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
        ("replay", &["replay_team_match"]),
        ("replay_frame", &["replay_frame_replay"]),
        ("replay_event", &["replay_event_replay"]),
        ("board", &["board_team"]),
        ("board_version", &["board_version_unique"]),
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

#[test]
fn link_keys_parse_only_matching_ids() {
    use migrations::link_key;
    assert_eq!(link_key("draft:abc", "draft").as_deref(), Some("abc"));
    assert_eq!(link_key(" draft:abc ", "draft").as_deref(), Some("abc"));
    assert_eq!(link_key("draft:⟨a-b⟩", "draft").as_deref(), Some("a-b"));
    assert_eq!(link_key("game_plan:abc", "draft"), None);
    assert_eq!(link_key("draft:", "draft"), None);
    assert_eq!(link_key("", "draft"), None);
    assert_eq!(link_key("abc", "draft"), None);
}

#[tokio::test]
async fn string_ids_become_record_links() {
    let db = baseline_only().await;
    // Recreate the pre-M6 string types, then legacy rows.
    for (table, field, _) in migrations::LINK_FIELDS {
        db.query(format!(
            "DEFINE FIELD OVERWRITE {field} ON {table} TYPE option<string>;"
        ))
        .await
        .unwrap()
        .check()
        .unwrap();
    }
    db.query(
        "CREATE user:u SET username = 'u', email = 'u@example.com', password_hash = 'h'; \
         CREATE team:t SET name = 'T', region = 'EUW', created_by = user:u; \
         CREATE series:s SET name = 'Bo3', team = team:t, created_by = user:u, opponent_id = 'opponent:o'; \
         CREATE draft:d SET name = 'D', team = team:t, created_by = user:u, series_id = 'series:s'; \
         CREATE draft:e SET name = 'E', team = team:t, created_by = user:u, series_id = ''; \
         CREATE game_plan:p SET team = team:t, draft = 'draft:d', win_conditions = [], objective_priority = [], teamfight_strategy = 'x'; \
         CREATE game_plan:q SET team = team:t, draft = 'not an id', win_conditions = [], objective_priority = [], teamfight_strategy = 'x'; \
         CREATE post_game_learning:r SET team = team:t, game_plan_id = 'game_plan:p', draft_id = 'draft:d', what_went_well = [], improvements = [], action_items = [], created_by = user:u; \
         CREATE action_item:a SET team = team:t, text = 'x', source_review = 'post_game_learning:r'; \
         CREATE checklist_template:c SET team = team:t, name = 'C'; \
         CREATE checklist_instance:i SET team = team:t, game_plan_id = 'game_plan:p', template_id = 'checklist_template:c';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();

    migrations::run(&db).await.unwrap();

    let check = |q: &'static str| {
        let db = &db;
        async move {
            let mut r = db.query(q).await.unwrap();
            let v: Value = r.take(0).unwrap();
            format!("{v:?}")
        }
    };
    // Each converted field now compares equal to a record, not a string.
    for (q, expect_rows) in [
        ("SELECT id FROM draft WHERE series_id = series:s", 1),
        ("SELECT id FROM draft WHERE series_id = NONE", 1),
        ("SELECT id FROM game_plan WHERE draft = draft:d", 1),
        ("SELECT id FROM game_plan WHERE draft = NONE", 1),
        ("SELECT id FROM post_game_learning WHERE game_plan_id = game_plan:p AND draft_id = draft:d", 1),
        ("SELECT id FROM series WHERE opponent_id = opponent:o", 1),
        ("SELECT id FROM action_item WHERE source_review = post_game_learning:r", 1),
        ("SELECT id FROM checklist_instance WHERE game_plan_id = game_plan:p AND template_id = checklist_template:c", 1),
    ] {
        let out = check(q).await;
        assert_eq!(out.matches("RecordId(").count(), expect_rows, "{q}: {out}");
    }
    let plan = info_for_table(&db, "game_plan").await;
    assert!(
        plan.contains("DEFINE FIELD draft ON game_plan TYPE none | record<draft>"),
        "{plan}"
    );

    // The models still see string IDs.
    let plans = db::list_game_plans(&db, "team:t").await.unwrap();
    let p = plans
        .iter()
        .find(|p| p.id.as_deref() == Some("game_plan:p"))
        .unwrap();
    assert_eq!(p.draft_id.as_deref(), Some("draft:d"));
    let reviews = db::list_post_game_learnings(&db, "team:t").await.unwrap();
    assert_eq!(reviews[0].game_plan_id.as_deref(), Some("game_plan:p"));
    assert_eq!(reviews[0].draft_id.as_deref(), Some("draft:d"));
}
