//! M8 account export and deletion: the export holds every kind of personal
//! data and nothing of other users; deletion leaves no row that points at
//! the user, keeps team records under the tombstone, settles the team
//! question, and ends sessions. Leadership hand-over and the hidden player
//! card are here too.

use std::collections::HashMap;

use lol_team_companion::models::share::ShareKind;
use lol_team_companion::server::account::{self, Treatment, FORMER_MEMBER_ID, USER_LINKS};
use lol_team_companion::server::db::{self, DbError};
use lol_team_companion::server::session_store::SurrealSessionStore;
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};

struct World {
    /// The member whose account is exported or deleted.
    alice: String,
    /// Leader of alice's team.
    lena: String,
    team: String,
    /// Someone in another team, whose data must never show up.
    otto: String,
}

async fn world(db: &Surreal<Db>) -> World {
    let lena = db::create_user(db, "lena".into(), "lena@example.com".into(), "h".into())
        .await
        .unwrap();
    let alice = db::create_user(db, "alice".into(), "alice@example.com".into(), "h".into())
        .await
        .unwrap();
    let otto = db::create_user(db, "otto".into(), "otto@example.com".into(), "h".into())
        .await
        .unwrap();
    let team = db::create_team(db, &lena, "Northgate".into(), "EUW".into())
        .await
        .unwrap();
    db::join_team(db, &alice, &team).await.unwrap();
    let other = db::create_team(db, &otto, "Riverside".into(), "EUW".into())
        .await
        .unwrap();
    seed_rows(db, &alice, &lena, &team, "a").await;
    // Otto has the same kinds of rows, in his own team.
    seed_rows(db, &otto, &otto, &other, "o").await;
    World {
        alice,
        lena,
        team,
        otto,
    }
}

/// One row in every table that links a user, for `user` in `team` (led by
/// `leader`). `tag` keeps fixed ids and unique values apart.
async fn seed_rows(db: &Surreal<Db>, user: &str, leader: &str, team: &str, tag: &str) {
    let q = "\
        LET $u = type::record('user', $u_key); \
        LET $l = type::record('user', $l_key); \
        LET $t = type::record('team', $t_key); \
        LET $name = (SELECT VALUE username FROM ONLY $u); \
        CREATE type::record('match', $tag) SET match_id = 'EUW1_' + $tag, queue_id = 420, game_duration = 1800, synced_by = $u, team_id = $t; \
        CREATE player_match SET match = type::record('match', $tag), user = $u, champion = 'Ahri-' + $tag, kills = 1, deaths = 2, assists = 3, cs = 100, vision_score = 10, damage = 1000, win = true; \
        CREATE type::record('draft', $tag) SET name = 'Draft ' + $tag, team = $t, created_by = $u; \
        CREATE type::record('game_plan', $tag) SET name = 'Plan ' + $tag, team = $t, win_conditions = [], objective_priority = [], teamfight_strategy = '', created_by = $u; \
        CREATE type::record('post_game_learning', $tag) SET team = $t, what_went_well = [], improvements = [], action_items = [], created_by = $u; \
        CREATE champion_pool SET user = $u, champion = 'Pool-' + $tag, role = 'mid'; \
        CREATE champion_note SET user = $u, champion = 'Ahri', role = 'mid', note_type = 'general', title = 'Note ' + $tag; \
        CREATE draft_tree SET name = 'Tree ' + $tag, team = $t, created_by = $u; \
        CREATE series SET name = 'Series ' + $tag, team = $t, created_by = $u; \
        CREATE team_note SET team = $t, author = $u, author_name = $name, content = 'note text ' + $tag, edited_by_name = $name; \
        CREATE invite SET code = 'ACCOUNT-TEST-' + $tag, created_by = $u; \
        CREATE ranked_snapshot SET user = $u, queue_type = 'RANKED_SOLO_5x5', tier = 'GOLD', division = 'II', lp = 50, wins = 10, losses = 9; \
        CREATE personal_learning SET user = $u, title = 'Learning ' + $tag, learning_type = 'general', what_happened = 'x', what_i_learned = 'y', next_time = 'z'; \
        CREATE personal_goal SET user = $u, goal_type = 'rank', target_value = 'plat'; \
        CREATE bug_report SET user = $u, page_url = '/draft', element_label = 'Save button', description = 'report text ' + $tag, category = 'bug', triaged_by = $u, public_title = 'Public ' + $tag; \
        CREATE type::record('board', $tag) SET team = $t, name = 'Board ' + $tag, created_by = $u; \
        CREATE board_version SET board = type::record('board', $tag), version = 1, scene = {}, author = $u; \
        CREATE type::record('champion_trait', 'Trait' + $tag) SET champion = 'Trait' + $tag, roles = ['mid'], damage_type = 'magic', damage_profile = 'burst', range_class = 'long', engage = 'none', disengage = 'none', peel = 'none', frontline = 'none', cc = 'light', wave_clear = 'ok', scaling = 'mid', mobility = 'medium', styles = [], confidence = 'normal', source = 'maintainer', edited_by = $u, edited_by_name = $name; \
        CREATE team_relation SET team = $t, a = 'Ahri', b = 'Zed', kind = 'counter', weight = 2, reason = 'pair note ' + $tag, author = $u, author_name = $name; \
        CREATE scheduled_match SET team = $t, starts_at = time::now(), kind = 'scrim', format = 'bo1', brief_read_by = [$u, $l], created_by = $u; \
        CREATE share_link SET token = 'card' + $tag + 'aaaaaaaaaaaaaaaaaaaaaaaaaaa', team = $t, kind = 'player', target = $u, created_by = $l, expires_at = time::now() + 7d; \
        CREATE share_link SET token = 'plan' + $tag + 'aaaaaaaaaaaaaaaaaaaaaaaaaaa', team = $t, kind = 'game_plan', target = type::record('game_plan', $tag), created_by = $u, expires_at = time::now() + 7d; \
        CREATE type::record('survey', $tag) SET question = 'Question ' + $tag, kind = 'scale', trigger = 'review_saved', wording_hash = 'h', approved_by = $u, created_by = $u, updated_by = $u; \
        CREATE survey_prompt SET user = $u, survey = type::record('survey', $tag), state = 'answered', answer_scale = 4, answer_text = 'answer ' + $tag; \
        CREATE type::record('proposal', $tag) SET title = 'Idea ' + $tag, published_by = $u, created_by = $u, updated_by = $u; \
        CREATE proposal_vote SET proposal = type::record('proposal', $tag), user = $u; \
        CREATE ledger_month SET month = IF $tag = 'a' { '2026-08' } ELSE { '2026-09' }, updated_by = $u; \
        CREATE site_text SET slug = 'account-test-' + $tag, body = 'b', approved_by = $u, updated_by = $u; \
        CREATE join_request SET team = $t, user = $u, status = 'accepted'; \
        CREATE action_item SET team = $t, text = 'do it ' + $tag, assigned_to = $name;";
    db.query(q)
        .bind(("u_key", db::record_key(user, "user")))
        .bind(("l_key", db::record_key(leader, "user")))
        .bind(("t_key", db::record_key(team, "team")))
        .bind(("tag", tag.to_string()))
        .await
        .unwrap()
        .check()
        .unwrap();
}

/// How many rows point at `user` through each listed user link.
async fn links_to(db: &Surreal<Db>, user: &str) -> HashMap<String, i64> {
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let mut out = HashMap::new();
    for (table, field, treatment) in USER_LINKS {
        // Compile-time constants from USER_LINKS; the user is bound.
        let q = if *treatment == Treatment::RemoveFromArray {
            format!("SELECT count() AS n FROM {table} WHERE $u IN {field} GROUP ALL")
        } else {
            format!("SELECT count() AS n FROM {table} WHERE {field} = $u GROUP ALL")
        };
        let mut r = db
            .query(format!("LET $u = type::record('user', $k); {q}"))
            .bind(("k", db::record_key(user, "user")))
            .await
            .unwrap();
        let n: Option<N> = r.take(1).unwrap();
        out.insert(format!("{table}.{field}"), n.map(|n| n.n).unwrap_or(0));
    }
    out
}

async fn count(db: &Surreal<Db>, q: &str) -> i64 {
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let mut r = db.query(q).await.unwrap();
    let n: Option<N> = r.take(0).unwrap();
    n.map(|n| n.n).unwrap_or(0)
}

#[tokio::test]
async fn the_seed_covers_every_user_link() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let links = links_to(&db, &w.alice).await;
    for (table, field, treatment) in USER_LINKS {
        if *treatment == Treatment::Leader {
            continue; // alice is not a leader
        }
        assert!(
            links[&format!("{table}.{field}")] > 0,
            "the fixture has no {table}.{field} row for alice"
        );
    }
}

#[tokio::test]
async fn export_holds_every_section_and_nothing_of_others() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let doc = account::export_user_data(&db, &w.alice).await.unwrap();
    let data = doc["data"].as_object().unwrap();
    for name in account::export_section_names() {
        let v = &data[name];
        let filled = match v {
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Object(o) => !o.is_empty(),
            _ => false,
        };
        assert!(filled, "export section {name} is empty: {v}");
    }
    assert_eq!(data["account"]["email"], "alice@example.com");
    assert!(data["account"].get("password_hash").is_none());
    let text = doc.to_string();
    for own in [
        "Pool-a",
        "Learning a",
        "report text a",
        "answer a",
        "note text a",
        "pair note a",
        "Idea a",
        "Draft a",
    ] {
        assert!(text.contains(own), "{own} missing from the export");
    }
    for foreign in [
        "otto",
        "lena@example.com",
        "Pool-o",
        "Learning o",
        "report text o",
        "answer o",
        "note text o",
        "Draft o",
    ] {
        assert!(
            !text.contains(foreign),
            "{foreign} leaked into alice's export"
        );
    }
    assert!(!text.contains("\"h\""), "a password hash value leaked");
}

#[tokio::test]
async fn deletion_leaves_no_row_pointing_at_the_user() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    // Another team uses the free-text name "alice" for someone else: the
    // deletion must not touch that team's rows (review fix).
    db.query(
        "LET $other = (SELECT VALUE id FROM ONLY team WHERE name = 'Riverside' LIMIT 1); \
         CREATE action_item SET team = $other, text = 'other team item', assigned_to = 'alice'; \
         CREATE team_note SET team = $other, author = user:former_member, author_name = 'x', content = 'other team note', edited_by_name = 'alice';",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    let out = account::delete_account(&db, &w.alice).await.unwrap();
    assert!(!out.team_deleted);
    assert_eq!(count(&db, "SELECT count() AS n FROM action_item WHERE text = 'other team item' AND assigned_to = 'alice' GROUP ALL").await, 1);
    assert_eq!(count(&db, "SELECT count() AS n FROM team_note WHERE content = 'other team note' AND edited_by_name = 'alice' GROUP ALL").await, 1);

    for (link, n) in links_to(&db, &w.alice).await {
        assert_eq!(n, 0, "{link} still points at the deleted user");
    }
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM user WHERE email = 'alice@example.com' GROUP ALL"
        )
        .await,
        0
    );
    // Personal rows are gone, team records stay under the tombstone.
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM champion_pool WHERE champion = 'Pool-a' GROUP ALL"
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM draft WHERE created_by = user:former_member GROUP ALL"
        )
        .await,
        1
    );
    assert_eq!(
        count(&db, "SELECT count() AS n FROM team_note WHERE author = user:former_member AND author_name = 'Former member' AND edited_by_name = 'Former member' GROUP ALL").await,
        1
    );
    assert_eq!(
        count(&db, "SELECT count() AS n FROM bug_report WHERE user = user:former_member AND public_title = 'Public a' AND element_label = '' AND !string::contains(description, 'report text') GROUP ALL").await,
        1
    );
    assert_eq!(count(&db, "SELECT count() AS n FROM action_item WHERE text = 'do it a' AND assigned_to = NONE GROUP ALL").await, 1);
    // Lena stays in the "brief read" list; the card link to alice is gone.
    assert_eq!(count(&db, "SELECT count() AS n FROM scheduled_match WHERE array::len(brief_read_by) = 1 GROUP ALL").await, 1);
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM share_link WHERE kind = 'player' GROUP ALL"
        )
        .await,
        1,
        "only otto's card link remains"
    );

    // Otto and the team are untouched.
    let otto_links = links_to(&db, &w.otto).await;
    assert!(otto_links["champion_pool.user"] > 0 && otto_links["bug_report.user"] > 0);
    assert!(db::is_team_member(&db, &w.lena, &w.team).await.unwrap());

    // The tombstone cannot sign in or be deleted.
    assert!(matches!(
        account::delete_account(&db, FORMER_MEMBER_ID).await,
        Err(DbError::NotFound)
    ));
    // Deleting again is "not found".
    assert!(matches!(
        account::delete_account(&db, &w.alice).await,
        Err(DbError::NotFound)
    ));
}

#[tokio::test]
async fn a_leader_hands_over_first_or_deletes_a_team_of_one() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let err = account::delete_account(&db, &w.lena).await.unwrap_err();
    assert!(
        matches!(err, DbError::Invalid(ref m) if m.contains("Hand leadership")),
        "{err:?}"
    );
    assert!(db::is_team_leader(&db, &w.lena, &w.team).await.unwrap());

    // Hand-over to an outsider is "not found"; to a member works.
    assert!(matches!(
        account::hand_over_leadership(&db, &w.team, &w.otto).await,
        Err(DbError::NotFound)
    ));
    account::hand_over_leadership(&db, &w.team, &w.alice)
        .await
        .unwrap();
    assert!(db::is_team_leader(&db, &w.alice, &w.team).await.unwrap());
    assert!(!db::is_team_leader(&db, &w.lena, &w.team).await.unwrap());

    // Lena is a plain member now and can go; the team stays.
    let out = account::delete_account(&db, &w.lena).await.unwrap();
    assert!(!out.team_deleted);
    assert!(db::is_team_member(&db, &w.alice, &w.team).await.unwrap());

    // Alice is now the only member: her deletion takes the team with it.
    let out = account::delete_account(&db, &w.alice).await.unwrap();
    assert!(out.team_deleted);
    for table in [
        "team",
        "draft",
        "game_plan",
        "team_note",
        "share_link",
        "board",
    ] {
        let n = count(
            &db,
            &format!("SELECT count() AS n FROM {table} WHERE team = type::record('team', '{}') OR id = type::record('team', '{}') GROUP ALL",
                db::record_key(&w.team, "team"), db::record_key(&w.team, "team")),
        )
        .await;
        assert_eq!(n, 0, "{table} rows of the deleted team remain");
    }
    // Otto's team is untouched.
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM draft WHERE name = 'Draft o' GROUP ALL"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn deleting_ends_the_users_sessions_only() {
    use tower_sessions_core::session::{Id, Record};
    use tower_sessions_core::SessionStore;
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let store = SurrealSessionStore::new(db.clone(), "sessions".into());
    let expiry = tower_sessions::cookie::time::OffsetDateTime::now_utc()
        + tower_sessions::cookie::time::Duration::days(1);
    for user in [&w.alice, &w.alice, &w.otto] {
        let mut data = HashMap::new();
        data.insert(
            "axum-login.data".to_string(),
            serde_json::json!({ "user_id": user, "auth_hash": [1, 2, 3] }),
        );
        let mut record = Record {
            id: Id::default(),
            data,
            expiry_date: expiry,
        };
        store.create(&mut record).await.unwrap();
    }
    assert_eq!(store.delete_for_user(&w.alice).await.unwrap(), 2);
    assert_eq!(
        count(&db, "SELECT count() AS n FROM sessions GROUP ALL").await,
        1
    );
}

#[tokio::test]
async fn a_hidden_card_cannot_be_shared_and_old_links_are_revoked() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let link = db::create_share_link(&db, &w.team, &w.lena, ShareKind::Player, &w.alice, 7)
        .await
        .unwrap();
    account::set_share_card_hidden(&db, &w.alice, true)
        .await
        .unwrap();
    assert!(account::share_card_hidden(&db, &w.alice).await.unwrap());
    let err = db::create_share_link(&db, &w.team, &w.lena, ShareKind::Player, &w.alice, 7)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::Invalid(_)), "{err:?}");
    assert!(db::resolve_share_token(&db, &link.token)
        .await
        .unwrap()
        .is_none());
    let card = db::player_card(&db, &w.team, &w.alice).await.unwrap();
    assert!(card.share_hidden);
    // Turned back on: sharing works again (old links stay revoked).
    account::set_share_card_hidden(&db, &w.alice, false)
        .await
        .unwrap();
    db::create_share_link(&db, &w.team, &w.lena, ShareKind::Player, &w.alice, 7)
        .await
        .unwrap();
}
