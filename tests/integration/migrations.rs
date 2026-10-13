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
    let db = crate::common::open_with_retry(|| Surreal::new::<Mem>(())).await;
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
        (
            "team_member",
            &["team_member_team", "team_member_user_unique"],
        ),
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
        (
            "bug_report",
            &["bug_report_board_status", "bug_report_user"],
        ),
        (
            "survey",
            &["survey_seed_key_unique", "survey_trigger_status"],
        ),
        (
            "survey_prompt",
            &[
                "survey_prompt_user",
                "survey_prompt_unique",
                "survey_prompt_user_shown",
                "survey_prompt_survey_state",
            ],
        ),
        ("proposal", &["proposal_seed_key_unique", "proposal_status"]),
        (
            "proposal_vote",
            &[
                "proposal_vote_unique",
                "proposal_vote_proposal",
                "proposal_vote_user",
            ],
        ),
        (
            "ledger_month",
            &["ledger_month_unique", "ledger_month_published"],
        ),
        ("site_text", &["site_text_slug_unique"]),
        ("user_guidance", &["user_guidance_unique"]),
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
         CREATE user:eve1 SET username = 'Eve', email = 'Eve@Example.com', password_hash = 'h', created_at = d'2024-01-01T00:00:00Z'; \
         CREATE user:eve2 SET username = 'EveToo', email = 'EVE@example.com', password_hash = 'h', created_at = d'2024-02-01T00:00:00Z'; \
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
    // When none of them is lowercase, the oldest is lowercased so one account
    // can still sign in (login matches the lowercase form exactly).
    assert_eq!(get("eve1").await.email, "eve@example.com");
    assert_eq!(get("eve2").await.email, "EVE@example.com");

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

/// An older image started on a newer database must stop before it writes
/// anything: the version check runs before the `schema.surql` baseline (M8).
#[tokio::test]
async fn a_newer_database_is_refused_before_the_baseline_is_applied() {
    let db = crate::common::open_with_retry(|| Surreal::new::<Mem>(())).await;
    // A database from a future build: one recorded migration, one table.
    db.query(
        "CREATE schema_migration SET version = 9999, name = 'future', applied_at = time::now(); \
         DEFINE TABLE future_only SCHEMALESS;",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    let err = db::prepare(&db).await.unwrap_err();
    assert!(err.to_string().contains("9999"), "{err}");
    let mut r = db
        .query("RETURN object::keys((INFO FOR DB).tables)")
        .await
        .unwrap();
    let tables: Vec<String> = r.take(0).unwrap();
    assert!(tables.iter().any(|t| t == "future_only"), "{tables:?}");
    assert!(
        !tables.iter().any(|t| t == "user" || t == "team"),
        "the baseline was applied to a newer database: {tables:?}"
    );
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

#[tokio::test]
async fn extra_memberships_are_resolved_before_the_unique_index() {
    let db = baseline_only().await;
    db.query(
        "CREATE user:a SET username = 'a', email = 'a@example.com', password_hash = 'h'; \
         CREATE user:b SET username = 'b', email = 'b@example.com', password_hash = 'h'; \
         CREATE team:one SET name = 'One', region = 'EUW', created_by = user:b; \
         CREATE team:two SET name = 'Two', region = 'EUW', created_by = user:a; \
         CREATE team_member:m1 SET team = team:one, user = user:a, role = 'sub'; \
         CREATE team_member:m2 SET team = team:two, user = user:a, role = 'sub'; \
         CREATE team_member:m3 SET team = team:one, user = user:b, role = 'sub'; \
         CREATE team_member:m4 SET team = team:two, user = user:b, role = 'sub';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    migrations::run(&db).await.unwrap();

    // a leads team two: that membership stays. b leads team one.
    assert_eq!(
        db::get_user_team_id(&db, "user:a")
            .await
            .unwrap()
            .as_deref(),
        Some("team:two")
    );
    assert_eq!(
        db::get_user_team_id(&db, "user:b")
            .await
            .unwrap()
            .as_deref(),
        Some("team:one")
    );
    let info = info_for_table(&db, "team_member").await;
    assert!(info.contains("team_member_user_unique"), "{info}");
    assert!(!info.contains("team_member_user ON"), "{info}");

    // The database now refuses a second membership on its own.
    match db::join_team(&db, "user:a", "team:one").await {
        Err(DbError::Invalid(msg)) => assert!(msg.contains("already belongs"), "{msg}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[tokio::test]
async fn the_user_theme_field_is_dropped() {
    let db = baseline_only().await;
    // The pre-M2 shape: the field was defined and every user carried a value.
    db.query(
        "DEFINE FIELD theme ON user TYPE string DEFAULT 'demacia'; \
         CREATE user:themed SET username = 'Theo', email = 'theo@example.com', password_hash = 'h', theme = 'pandemonium';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    assert!(info_for_table(&db, "user").await.contains("theme"));

    migrations::run(&db).await.unwrap();

    assert!(
        !info_for_table(&db, "user").await.contains("theme"),
        "the field definition is gone"
    );
    let mut r = db
        .query("SELECT VALUE theme FROM user:themed")
        .await
        .unwrap();
    let v: Value = r.take(0).unwrap();
    let shown = format!("{v:?}");
    assert!(!shown.contains("pandemonium"), "the value is gone: {shown}");

    // A user created afterwards gets no theme either.
    db.query(
        "CREATE user:fresh SET username = 'Fay', username_lc = 'fay', email = 'fay@example.com', password_hash = 'h'",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    let mut r = db
        .query("SELECT VALUE theme FROM user:fresh")
        .await
        .unwrap();
    let v: Value = r.take(0).unwrap();
    assert!(!format!("{v:?}").contains("demacia"));
}

#[tokio::test]
async fn the_user_mode_field_is_dropped() {
    let db = baseline_only().await;
    // The pre-M9 shape: every user carried the old Solo/Team switch.
    db.query(
        "DEFINE FIELD mode ON user TYPE string DEFAULT 'solo'; \
         CREATE user:moded SET username = 'Mo', email = 'mo@example.com', password_hash = 'h', mode = 'team';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    assert!(info_for_table(&db, "user").await.contains("mode"));

    migrations::run(&db).await.unwrap();

    let info = info_for_table(&db, "user").await;
    assert!(
        !info.contains(" mode ON"),
        "the field definition is gone: {info}"
    );
    let mut r = db.query("SELECT VALUE mode FROM user:moded").await.unwrap();
    let v: Value = r.take(0).unwrap();
    assert!(
        !format!("{v:?}").contains("team"),
        "the value is gone: {v:?}"
    );
}

#[tokio::test]
async fn roster_places_roles_and_dead_links_are_migrated() {
    let db = baseline_only().await;
    // The pre-M5 shape: per-page view preferences on users, mixed role
    // spellings, "sub" for everyone not starting, a plan and a review whose
    // draft was deleted, and pool roles in several spellings.
    db.query(
        "DEFINE FIELD draft_mode ON user TYPE string DEFAULT 'auto'; \
         DEFINE FIELD solo_mode ON user TYPE string DEFAULT 'auto'; \
         DEFINE FIELD team_dashboard_mode ON user TYPE string DEFAULT 'auto'; \
         CREATE user:lead SET username = 'Lead', email = 'lead@example.com', password_hash = 'h', draft_mode = 'war-table'; \
         CREATE user:a SET username = 'A', email = 'a@example.com', password_hash = 'h'; \
         CREATE user:b SET username = 'B', email = 'b@example.com', password_hash = 'h'; \
         CREATE user:c SET username = 'C', email = 'c@example.com', password_hash = 'h'; \
         CREATE user:d SET username = 'D', email = 'd@example.com', password_hash = 'h'; \
         CREATE team:t SET name = 'T', region = 'EUW', created_by = user:lead; \
         CREATE team_member:lead SET team = team:t, user = user:lead, role = 'unassigned', roster_type = 'sub'; \
         CREATE team_member:a SET team = team:t, user = user:a, role = 'ADC', roster_type = 'starter'; \
         CREATE team_member:b SET team = team:t, user = user:b, role = 'Jungle', roster_type = 'sub'; \
         CREATE team_member:c SET team = team:t, user = user:c, role = 'coach', roster_type = 'sub'; \
         CREATE team_member:d SET team = team:t, user = user:d, role = 'sub'; \
         CREATE draft:kept SET name = 'Kept', team = team:t, created_by = user:lead; \
         CREATE game_plan:dead SET team = team:t, draft = draft:gone, win_conditions = [], objective_priority = [], teamfight_strategy = ''; \
         CREATE game_plan:live SET team = team:t, draft = draft:kept, win_conditions = [], objective_priority = [], teamfight_strategy = ''; \
         CREATE post_game_learning:dead SET team = team:t, draft_id = draft:gone, what_went_well = [], improvements = [], action_items = [], created_by = user:lead; \
         CREATE champion_pool:p1 SET user = user:a, champion = 'Jinx', role = 'ADC', created_at = d'2026-01-01T00:00:00Z'; \
         CREATE champion_pool:p2 SET user = user:a, champion = 'Jinx', role = 'bot', created_at = d'2026-02-01T00:00:00Z'; \
         CREATE champion_pool:p3 SET user = user:a, champion = 'Lee Sin', role = 'jgl'; \
         CREATE champion_note:n1 SET user = user:a, champion = 'Jinx', role = 'Bottom', note_type = 'tip', title = 't';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();

    migrations::run(&db).await.unwrap();

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Place {
        role: String,
        roster_type: String,
    }
    let place = |key: &'static str| {
        let db = db.clone();
        async move {
            let mut r = db
                .query("SELECT role, roster_type FROM type::record('team_member', $k)")
                .bind(("k", key))
                .await
                .unwrap();
            let p: Option<Place> = r.take(0).unwrap();
            let p = p.unwrap();
            (p.roster_type, p.role)
        }
    };
    assert_eq!(place("lead").await, ("member".into(), "unassigned".into()));
    assert_eq!(place("a").await, ("starter".into(), "bot".into()));
    assert_eq!(place("b").await, ("substitute".into(), "jungle".into()));
    assert_eq!(place("c").await, ("coach".into(), "unassigned".into()));
    assert_eq!(place("d").await, ("member".into(), "unassigned".into()));

    // The fields are tightened: an unknown place is refused.
    assert!(db
        .query("UPDATE team_member:a SET roster_type = 'sub'")
        .await
        .unwrap()
        .check()
        .is_err());

    // The view preferences are gone.
    let info = info_for_table(&db, "user").await;
    assert!(!info.contains("draft_mode"), "{info}");
    assert!(!info.contains("solo_mode"), "{info}");
    assert!(!info.contains("team_dashboard_mode"), "{info}");

    // Dead draft links became marks; live links stay.
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Link {
        draft: Option<surrealdb::types::RecordId>,
        draft_deleted: Option<bool>,
    }
    let mut r = db
        .query("SELECT draft, draft_deleted FROM game_plan:dead; SELECT draft, draft_deleted FROM game_plan:live; SELECT draft_id AS draft, draft_deleted FROM post_game_learning:dead;")
        .await
        .unwrap();
    let dead: Option<Link> = r.take(0).unwrap();
    let live: Option<Link> = r.take(1).unwrap();
    let review: Option<Link> = r.take(2).unwrap();
    let (dead, live, review) = (dead.unwrap(), live.unwrap(), review.unwrap());
    assert!(dead.draft.is_none() && dead.draft_deleted == Some(true));
    assert!(live.draft.is_some() && live.draft_deleted.is_none());
    assert!(review.draft.is_none() && review.draft_deleted == Some(true));

    // Pool roles in one spelling, the duplicate Jinx gone (oldest kept).
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Pool {
        id: surrealdb::types::RecordId,
        role: String,
    }
    let mut r = db
        .query("SELECT id, role FROM champion_pool ORDER BY id; SELECT VALUE role FROM champion_note:n1;")
        .await
        .unwrap();
    let pool: Vec<Pool> = r.take(0).unwrap();
    let note_role: Option<String> = r.take(1).unwrap();
    use surrealdb::types::ToSql;
    let pool: Vec<(String, String)> = pool.into_iter().map(|p| (p.id.to_sql(), p.role)).collect();
    assert_eq!(
        pool,
        vec![
            ("champion_pool:p1".to_string(), "bot".to_string()),
            ("champion_pool:p3".to_string(), "jungle".to_string()),
        ]
    );
    assert_eq!(note_role.as_deref(), Some("bot"));

    // Running again changes nothing.
    assert!(migrations::run(&db).await.unwrap().is_empty());
}

#[tokio::test]
async fn bug_report_statuses_are_mapped_to_v2() {
    let db = baseline_only().await;
    db.query(
        "DEFINE FIELD OVERWRITE status ON bug_report TYPE string DEFAULT 'open' \
             ASSERT $value IN ['open', 'triaged', 'closed']; \
         CREATE user:r SET username = 'R', email = 'r@example.com', password_hash = 'h'; \
         CREATE bug_report:o SET user = user:r, page_url = '/', element_label = 'x', description = 'd', category = 'bug', status = 'open'; \
         CREATE bug_report:t SET user = user:r, page_url = '/', element_label = 'x', description = 'd', category = 'bug', status = 'triaged'; \
         CREATE bug_report:c SET user = user:r, page_url = '/', element_label = 'x', description = 'd', category = 'wishlist', status = 'closed';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();

    migrations::run(&db).await.unwrap();

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Row {
        status: String,
        admin_note: Option<String>,
        status_changed_at: Option<surrealdb::types::Datetime>,
    }
    let row = |key: &'static str| {
        let db = db.clone();
        async move {
            let mut r = db
                .query("SELECT status, admin_note, status_changed_at FROM type::record('bug_report', $k)")
                .bind(("k", key))
                .await
                .unwrap();
            let row: Option<Row> = r.take(0).unwrap();
            row.unwrap()
        }
    };
    assert_eq!(row("o").await.status, "new");
    assert_eq!(row("t").await.status, "triaged");
    let closed = row("c").await;
    assert_eq!(closed.status, "triaged");
    assert_eq!(
        closed.admin_note.as_deref(),
        Some("Closed before statuses existed; set a final status.")
    );
    // Not shown to the reporter as an update (M7 review).
    assert!(closed.status_changed_at.is_none());

    let bad = db
        .query("CREATE bug_report SET user = user:r, page_url = '/', element_label = 'x', description = 'd', category = 'bug', status = 'open'")
        .await
        .unwrap()
        .check();
    assert!(bad.is_err(), "the old status is rejected");
    db.query("CREATE bug_report:fresh SET user = user:r, page_url = '/', element_label = 'x', description = 'd', category = 'bug'")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(row("fresh").await.status, "new");
    let info = info_for_table(&db, "bug_report").await;
    assert!(info.contains("wont_fix"), "{info}");
}

#[tokio::test]
async fn community_seed_content_is_inserted_once_and_unapproved() {
    let db = crate::common::test_db().await;

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Counts {
        proposals: i64,
        published: i64,
        surveys: i64,
        approved_surveys: i64,
        drafts: i64,
        texts: i64,
        approved_texts: i64,
        player_cards: Option<String>,
    }
    let counts = || {
        let db = db.clone();
        async move {
            let mut r = db
                .query(
                    "RETURN { \
                        proposals: count(SELECT id FROM proposal WHERE seed_key != NONE), \
                        published: count(SELECT id FROM proposal WHERE published = true), \
                        surveys: count(SELECT id FROM survey WHERE seed_key != NONE), \
                        approved_surveys: count(SELECT id FROM survey WHERE approved_hash != NONE), \
                        drafts: count(SELECT id FROM survey WHERE status = 'draft'), \
                        texts: count(SELECT id FROM site_text), \
                        approved_texts: count(SELECT id FROM site_text WHERE approved = true), \
                        player_cards: (SELECT VALUE shipped_in FROM proposal WHERE seed_key = 'player_cards')[0] \
                     }",
                )
                .await
                .unwrap();
            let c: Option<Counts> = r.take(0).unwrap();
            c.unwrap()
        }
    };
    let c = counts().await;
    assert_eq!(c.proposals, 17);
    assert_eq!(
        c.published, 0,
        "nothing reaches the board before an admin publishes it"
    );
    assert_eq!(c.surveys, 4);
    assert_eq!(c.drafts, 4);
    assert_eq!(c.approved_surveys, 0);
    assert_eq!(c.texts, 5);
    assert_eq!(c.approved_texts, 0);
    assert_eq!(
        c.player_cards.as_deref(),
        Some("milestone-m5-remaining-pages-navigation-mobile")
    );

    // Running the body again adds nothing; a deleted seed stays deleted
    // because the migration is recorded.
    migrations::community_seed_content(&db).await.unwrap();
    assert_eq!(counts().await.proposals, 17);
    db.query("DELETE proposal WHERE seed_key = 'light_theme'")
        .await
        .unwrap()
        .check()
        .unwrap();
    migrations::run(&db).await.unwrap();
    assert_eq!(counts().await.proposals, 16);
}

#[tokio::test]
async fn payment_fees_become_a_cost_line_and_old_labels_need_approval_again() {
    let db = baseline_only().await;
    // Before the polish pass: a month without fees and the eight-line
    // labels text, approved.
    db.query(
        "CREATE ledger_month:⟨2026-09⟩ SET month = '2026-09', support_cents = 2000, hosting_cents = 500; \
         CREATE site_text SET slug = 'ledger_labels', approved = true, approved_at = time::now(), \
             body = 'Support received\nHosting\nDomain\nAI tokens\nOther\nRunning costs\nTo the maintainer\nNot covered by support';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();

    migrations::run(&db).await.unwrap();

    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct Row {
        fees: i64,
        body: String,
        approved: bool,
    }
    let mut r = db
        .query(
            "RETURN { fees: ledger_month:⟨2026-09⟩.payment_fees_cents, \
                body: (SELECT VALUE body FROM site_text WHERE slug = 'ledger_labels')[0], \
                approved: (SELECT VALUE approved FROM site_text WHERE slug = 'ledger_labels')[0] }",
        )
        .await
        .unwrap();
    let row: Option<Row> = r.take(0).unwrap();
    let row = row.unwrap();
    assert_eq!(row.fees, 0);
    assert!(row.body.ends_with("Not covered by support\nPayment fees"));
    assert!(
        lol_team_companion::models::community::LedgerLabels::parse(&row.body).is_some(),
        "the migrated text parses as nine labels"
    );
    assert!(!row.approved, "a changed money text needs approval again");
    // Negative fees are refused from now on.
    assert!(db
        .query("UPDATE ledger_month:⟨2026-09⟩ SET payment_fees_cents = -1")
        .await
        .unwrap()
        .check()
        .is_err());
}
