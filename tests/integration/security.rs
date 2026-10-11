//! M1 security suite: cross-tenant isolation, invite gate, session expiry,
//! query-injection regression.
//!
//! Cross-tenant tests follow one shape: user A (team 1) owns a resource, user
//! B (team 2) tries to read/update/delete it through the scoped DB function
//! and must get "not found" (or `None` / an empty list) with A's data intact;
//! A's own call succeeds.

use crate::common;

use lol_team_companion::models::champion::ChampionNote;
use lol_team_companion::models::game_plan::{GamePlan, PostGameLearning};
use lol_team_companion::models::personal_learning::PersonalLearning;
use lol_team_companion::server::db::{self, DbError};
use surrealdb::{engine::local::Db, Surreal};

struct Tenants {
    a: String,
    team1: String,
    b: String,
    team2: String,
}

async fn tenants(db: &Surreal<Db>) -> Tenants {
    let a = db::create_user(db, "alice".into(), "alice@example.com".into(), "h".into())
        .await
        .unwrap();
    let b = db::create_user(db, "bob".into(), "bob@example.com".into(), "h".into())
        .await
        .unwrap();
    let team1 = db::create_team(db, &a, "Team One".into(), "EUW".into())
        .await
        .unwrap();
    let team2 = db::create_team(db, &b, "Team Two".into(), "EUW".into())
        .await
        .unwrap();
    Tenants { a, team1, b, team2 }
}

fn assert_not_found<T: std::fmt::Debug>(r: Result<T, DbError>, what: &str) {
    match r {
        Err(DbError::NotFound) => {}
        other => panic!("{what}: expected NotFound, got {other:?}"),
    }
}

async fn make_draft(db: &Surreal<Db>, team: &str, user: &str, name: &str) -> String {
    db::save_draft(
        db,
        team,
        user,
        name.into(),
        None,
        None,
        vec![],
        vec![],
        None,
        "blue".into(),
        vec![],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap()
}

fn plan(team: &str, name: &str) -> GamePlan {
    GamePlan {
        id: None,
        team_id: team.into(),
        draft_id: None,
        name: name.into(),
        our_champions: vec![],
        enemy_champions: vec![],
        win_conditions: vec![],
        objective_priority: vec![],
        teamfight_strategy: "front to back".into(),
        early_game: None,
        top_strategy: None,
        jungle_strategy: None,
        mid_strategy: None,
        bot_strategy: None,
        support_strategy: None,
        notes: None,
        win_condition_tag: None,
    }
}

// ---------------------------------------------------------------------------
// Cross-tenant: drafts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn draft_is_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let draft = make_draft(&db, &t.team1, &t.a, "A's draft").await;

    // Read
    assert!(db::get_draft_for_prefill(&db, &t.team2, &draft)
        .await
        .unwrap()
        .is_none());
    assert!(db::get_draft_for_prefill(&db, &t.team1, &draft)
        .await
        .unwrap()
        .is_some());
    assert!(db::list_drafts(&db, &t.team2).await.unwrap().is_empty());

    // Update
    let update = |team: String| {
        let db = db.clone();
        let draft = draft.clone();
        async move {
            db::update_draft(
                &db,
                &team,
                &draft,
                "renamed".into(),
                None,
                None,
                vec![],
                vec![],
                None,
                "red".into(),
                vec![],
                None,
                None,
                None,
                None,
            )
            .await
        }
    };
    assert_not_found(update(t.team2.clone()).await, "B updates A's draft");
    assert_eq!(
        db::list_drafts(&db, &t.team1).await.unwrap()[0].name,
        "A's draft"
    );
    update(t.team1.clone()).await.unwrap();
    assert_eq!(
        db::list_drafts(&db, &t.team1).await.unwrap()[0].name,
        "renamed"
    );

    // Delete
    assert_not_found(
        db::delete_draft(&db, &t.team2, &draft).await,
        "B deletes A's draft",
    );
    assert_eq!(db::list_drafts(&db, &t.team1).await.unwrap().len(), 1);
    db::delete_draft(&db, &t.team1, &draft).await.unwrap();
    assert!(db::list_drafts(&db, &t.team1).await.unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Cross-tenant: draft trees and nodes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tree_and_nodes_are_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let tree = db::create_draft_tree(&db, &t.team1, &t.a, "A tree".into(), None)
        .await
        .unwrap();
    let root = db::get_tree_nodes(&db, &t.team1, &tree).await.unwrap()[0]
        .id
        .clone()
        .unwrap();

    // Tree read/update/delete
    assert_not_found(
        db::get_tree_nodes(&db, &t.team2, &tree).await,
        "B reads A's tree",
    );
    assert_not_found(
        db::update_draft_tree(&db, &t.team2, &tree, "x".into(), None).await,
        "B renames A's tree",
    );
    assert_not_found(
        db::delete_draft_tree(&db, &t.team2, &tree).await,
        "B deletes A's tree",
    );

    // Node create/update/delete
    assert_not_found(
        db::create_tree_node(&db, &t.team2, &tree, Some(root.clone()), "B".into()).await,
        "B adds a node to A's tree",
    );
    assert_not_found(
        db::update_tree_node(&db, &t.team2, &root, "hacked".into(), None, false, vec![]).await,
        "B edits A's node",
    );
    assert_not_found(
        db::delete_tree_node(&db, &t.team2, &root).await,
        "B deletes A's node",
    );

    // B cannot graft A's node under B's own tree either.
    let b_tree = db::create_draft_tree(&db, &t.team2, &t.b, "B tree".into(), None)
        .await
        .unwrap();
    assert_not_found(
        db::create_tree_node(&db, &t.team2, &b_tree, Some(root.clone()), "graft".into()).await,
        "B uses A's node as parent",
    );

    // A's own operations work and A's data is intact.
    let nodes = db::get_tree_nodes(&db, &t.team1, &tree).await.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].label, "A tree");
    let child = db::create_tree_node(&db, &t.team1, &tree, Some(root.clone()), "child".into())
        .await
        .unwrap();
    db::update_tree_node(&db, &t.team1, &child, "child2".into(), None, false, vec![])
        .await
        .unwrap();
    db::delete_tree_node(&db, &t.team1, &child).await.unwrap();
    db::update_draft_tree(&db, &t.team1, &tree, "renamed".into(), None)
        .await
        .unwrap();
    db::delete_draft_tree(&db, &t.team1, &tree).await.unwrap();
    assert!(db::list_draft_trees(&db, &t.team1)
        .await
        .unwrap()
        .is_empty());
}

// ---------------------------------------------------------------------------
// Cross-tenant: game plans and checklists
// ---------------------------------------------------------------------------

#[tokio::test]
async fn game_plan_and_checklist_are_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let plan_id = db::save_game_plan(&db, plan(&t.team1, "A plan"), &t.a)
        .await
        .unwrap();

    // Plan read/update/delete
    assert!(db::get_game_plan(&db, &t.team2, &plan_id)
        .await
        .unwrap()
        .is_none());
    let mut edit = plan(&t.team2, "hacked");
    edit.id = Some(plan_id.clone());
    assert_not_found(
        db::update_game_plan(&db, &t.team2, edit.clone()).await,
        "B updates A's plan",
    );
    assert_not_found(
        db::delete_game_plan(&db, &t.team2, &plan_id).await,
        "B deletes A's plan",
    );
    let still = db::get_game_plan(&db, &t.team1, &plan_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(still.name, "A plan");
    // A's update keeps the plan in team 1 even though the payload says team 2.
    db::update_game_plan(&db, &t.team1, edit).await.unwrap();
    let updated = db::get_game_plan(&db, &t.team1, &plan_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.name, "hacked");
    assert_eq!(updated.team_id, t.team1);

    // Checklist template + instance
    let template =
        db::create_checklist_template(&db, &t.team1, "Pre-game".into(), vec!["wards".into()])
            .await
            .unwrap();
    assert_not_found(
        db::delete_checklist_template(&db, &t.team2, &template).await,
        "B deletes A's template",
    );
    let instance = db::create_checklist_instance(
        &db,
        &t.team1,
        Some(plan_id.clone()),
        Some(template.clone()),
        vec!["wards".into()],
    )
    .await
    .unwrap();
    assert!(db::get_checklist_for_plan(&db, &t.team2, &plan_id)
        .await
        .unwrap()
        .is_none());
    assert_not_found(
        db::update_checklist_checked(&db, &t.team2, &instance, vec![true]).await,
        "B ticks A's checklist",
    );
    assert_not_found(
        db::delete_checklist_instance(&db, &t.team2, &instance).await,
        "B deletes A's checklist",
    );
    db::update_checklist_checked(&db, &t.team1, &instance, vec![true])
        .await
        .unwrap();
    let cl = db::get_checklist_for_plan(&db, &t.team1, &plan_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cl.checked, vec![true]);
    db::delete_checklist_template(&db, &t.team1, &template)
        .await
        .unwrap();
    db::delete_game_plan(&db, &t.team1, &plan_id).await.unwrap();
}

// ---------------------------------------------------------------------------
// Cross-tenant: post-game reviews
// ---------------------------------------------------------------------------

#[tokio::test]
async fn review_is_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let review = PostGameLearning {
        id: None,
        team_id: t.team1.clone(),
        match_riot_id: None,
        game_plan_id: None,
        draft_id: None,
        what_went_well: vec!["vision".into()],
        improvements: vec![],
        action_items: vec![],
        open_notes: None,
        created_by: t.a.clone(),
        win_loss: Some("win".into()),
        rating: Some(4),
    };
    let id = db::save_post_game_learning(&db, review.clone())
        .await
        .unwrap();

    let mut edit = review.clone();
    edit.id = Some(id.clone());
    edit.team_id = t.team2.clone(); // a forged team must not move the review
    edit.open_notes = Some("hacked".into());
    assert_not_found(
        db::update_post_game_learning(&db, &t.team2, edit.clone()).await,
        "B updates A's review",
    );
    assert_not_found(
        db::delete_post_game_learning(&db, &t.team2, &id).await,
        "B deletes A's review",
    );
    assert!(db::list_post_game_learnings(&db, &t.team2)
        .await
        .unwrap()
        .is_empty());

    db::update_post_game_learning(&db, &t.team1, edit)
        .await
        .unwrap();
    let list = db::list_post_game_learnings(&db, &t.team1).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].open_notes.as_deref(), Some("hacked"));
    assert_eq!(list[0].team_id, t.team1, "team cannot be changed by update");
    db::delete_post_game_learning(&db, &t.team1, &id)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Cross-tenant: action items and team notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn action_item_is_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let item = db::create_action_item(&db, &t.team1, "ward more".into(), None, None, None)
        .await
        .unwrap();

    assert_not_found(
        db::update_action_item_status(&db, &t.team2, &item, "done".into()).await,
        "B closes A's item",
    );
    assert_not_found(
        db::update_action_item(&db, &t.team2, &item, "x".into(), None, None, None).await,
        "B edits A's item",
    );
    assert_not_found(
        db::delete_action_item(&db, &t.team2, &item).await,
        "B deletes A's item",
    );
    let items = db::list_action_items(&db, &t.team1).await.unwrap();
    assert_eq!(items[0].text, "ward more");
    assert_eq!(items[0].status, "open");

    db::update_action_item(
        &db,
        &t.team1,
        &item,
        "ward even more".into(),
        None,
        Some("2026-10-31".into()),
        Some("done".into()),
    )
    .await
    .unwrap();
    let items = db::list_action_items(&db, &t.team1).await.unwrap();
    assert_eq!(items[0].status, "done");
    assert!(items[0].resolved_at.is_some(), "done sets resolved_at");

    // Editing the text without a status keeps the item done.
    db::update_action_item(
        &db,
        &t.team1,
        &item,
        "ward the most".into(),
        None,
        Some("2026-10-31".into()),
        None,
    )
    .await
    .unwrap();
    let items = db::list_action_items(&db, &t.team1).await.unwrap();
    assert_eq!(items[0].text, "ward the most");
    assert_eq!(
        items[0].status, "done",
        "a text edit must not reopen the item"
    );
    assert!(items[0].resolved_at.is_some());
    assert_eq!(items[0].due.as_deref(), Some("2026-10-31"));
    db::update_action_item_status(&db, &t.team1, &item, "open".into())
        .await
        .unwrap();
    db::delete_action_item(&db, &t.team1, &item).await.unwrap();
}

#[tokio::test]
async fn action_item_source_review_must_belong_to_the_team() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let review = PostGameLearning {
        id: None,
        team_id: t.team2.clone(),
        match_riot_id: None,
        game_plan_id: None,
        draft_id: None,
        what_went_well: vec![],
        improvements: vec![],
        action_items: vec![],
        open_notes: None,
        created_by: t.b.clone(),
        win_loss: None,
        rating: None,
    };
    let theirs = db::save_post_game_learning(&db, review).await.unwrap();
    assert_not_found(
        db::create_action_item(&db, &t.team1, "x".into(), Some(theirs.clone()), None, None).await,
        "A links an item to B's review",
    );
    assert!(db::list_action_items(&db, &t.team1)
        .await
        .unwrap()
        .is_empty());
    // Their own review links fine.
    db::create_action_item(&db, &t.team2, "x".into(), Some(theirs), None, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn action_item_due_dates_are_checked() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let bad = db::create_action_item(
        &db,
        &t.team1,
        "x".into(),
        None,
        None,
        Some("next friday".into()),
    )
    .await;
    assert!(matches!(bad, Err(db::DbError::Invalid(_))), "{bad:?}");
    let item = db::create_action_item(
        &db,
        &t.team1,
        "x".into(),
        None,
        None,
        Some("2026-11-02".into()),
    )
    .await
    .unwrap();
    let items = db::list_action_items(&db, &t.team1).await.unwrap();
    assert_eq!(items[0].due.as_deref(), Some("2026-11-02"));
    // An empty date clears it.
    db::update_action_item(
        &db,
        &t.team1,
        &item,
        "x".into(),
        None,
        Some(String::new()),
        None,
    )
    .await
    .unwrap();
    let items = db::list_action_items(&db, &t.team1).await.unwrap();
    assert_eq!(items[0].due, None);
}

#[tokio::test]
async fn team_note_is_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let note = db::create_team_note(&db, &t.team1, &t.a, "alice", "secret plan".into())
        .await
        .unwrap();

    assert_not_found(
        db::update_team_note(&db, &t.team2, &note, "hacked".into()).await,
        "B edits A's note",
    );
    assert_not_found(
        db::toggle_pin_team_note(&db, &t.team2, &note, true).await,
        "B pins A's note",
    );
    assert_not_found(
        db::delete_team_note(&db, &t.team2, &note).await,
        "B deletes A's note",
    );
    assert!(db::list_team_notes(&db, &t.team2).await.unwrap().is_empty());
    let notes = db::list_team_notes(&db, &t.team1).await.unwrap();
    assert_eq!(notes[0].content, "secret plan");
    assert!(!notes[0].pinned);

    db::update_team_note(&db, &t.team1, &note, "new plan".into())
        .await
        .unwrap();
    db::toggle_pin_team_note(&db, &t.team1, &note, true)
        .await
        .unwrap();
    let notes = db::list_team_notes(&db, &t.team1).await.unwrap();
    assert_eq!(notes[0].content, "new plan");
    assert!(notes[0].pinned);
    db::delete_team_note(&db, &t.team1, &note).await.unwrap();
}

// ---------------------------------------------------------------------------
// Cross-tenant: opponents and opponent players
// ---------------------------------------------------------------------------

#[tokio::test]
async fn opponent_and_players_are_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let (opp, players) = db::create_opponent_with_players(
        &db,
        &t.team1,
        "Rivals".into(),
        None,
        vec![("mid".into(), Some("Faker#KR1".into()))],
    )
    .await
    .unwrap();
    let player = players[0].clone();

    // Opponent
    assert!(db::get_opponent(&db, &t.team2, &opp)
        .await
        .unwrap()
        .is_none());
    assert_not_found(
        db::update_opponent(&db, &t.team2, &opp, "x".into(), None).await,
        "B renames A's opponent",
    );
    assert_not_found(
        db::delete_opponent(&db, &t.team2, &opp).await,
        "B deletes A's opponent",
    );
    assert_not_found(
        db::add_opponent_player(&db, &t.team2, &opp, "x".into(), "top".into()).await,
        "B adds a player to A's opponent",
    );

    // Player
    assert_not_found(
        db::save_opponent_player_info(&db, &t.team2, &player, "x".into(), "mid".into(), None, None)
            .await,
        "B edits A's scouted player",
    );
    assert_not_found(
        db::update_opponent_player_champions(&db, &t.team2, &player, vec!["Zed".into()]).await,
        "B overwrites A's player champions",
    );
    assert_not_found(
        db::update_opponent_player_intel(&db, &t.team2, &player, None, vec![], None, None).await,
        "B overwrites A's player intel",
    );
    assert_not_found(
        db::delete_opponent_player(&db, &t.team2, &player).await,
        "B deletes A's scouted player",
    );
    assert_not_found(
        db::ensure_team_opponent_player(&db, &t.team2, &player).await,
        "ownership check for B",
    );

    // A's view is intact, and A's own calls work.
    let (o, ps) = db::get_opponent(&db, &t.team1, &opp)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(o.name, "Rivals");
    assert_eq!(ps.len(), 1);
    db::update_opponent_player_champions(&db, &t.team1, &player, vec!["Ahri".into()])
        .await
        .unwrap();
    db::add_opponent_player(&db, &t.team1, &opp, "Sub".into(), "top".into())
        .await
        .unwrap();
    db::delete_opponent_player(&db, &t.team1, &player)
        .await
        .unwrap();
    db::update_opponent(&db, &t.team1, &opp, "Rivals 2".into(), None)
        .await
        .unwrap();
    db::delete_opponent(&db, &t.team1, &opp).await.unwrap();
    assert!(db::list_opponents(&db, &t.team1).await.unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// User-scoped: personal learnings and champion notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn personal_learning_is_private_to_its_owner() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let learning = PersonalLearning {
        id: None,
        user_id: t.a.clone(),
        title: "Private".into(),
        learning_type: "general".into(),
        champion: None,
        opponent: None,
        what_happened: "x".into(),
        what_i_learned: "y".into(),
        next_time: "z".into(),
        tags: vec![],
        win_loss: None,
        match_riot_id: None,
        game_timestamp_ms: None,
        event_name: None,
        created_at: None,
    };
    let id = db::create_personal_learning(&db, learning.clone())
        .await
        .unwrap();

    assert!(db::get_personal_learning(&db, &t.b, &id)
        .await
        .unwrap()
        .is_none());
    assert!(db::list_personal_learnings(&db, &t.b)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        db::list_personal_learnings(&db, &t.a).await.unwrap().len(),
        1
    );
    let mut edit = learning.clone();
    edit.id = Some(id.clone());
    edit.user_id = t.b.clone();
    edit.title = "hacked".into();
    assert_not_found(
        db::update_personal_learning(&db, &t.b, edit.clone()).await,
        "B edits A's learning",
    );
    assert_not_found(
        db::delete_personal_learning(&db, &t.b, &id).await,
        "B deletes A's learning",
    );

    let mine = db::get_personal_learning(&db, &t.a, &id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mine.title, "Private");
    db::update_personal_learning(&db, &t.a, edit).await.unwrap();
    let mine = db::get_personal_learning(&db, &t.a, &id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mine.title, "hacked");
    assert_eq!(mine.user_id, t.a, "owner cannot be changed by update");
    db::delete_personal_learning(&db, &t.a, &id).await.unwrap();
}

#[tokio::test]
async fn champion_note_is_private_to_its_owner() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let note = ChampionNote {
        id: None,
        user_id: t.a.clone(),
        champion: "Ahri".into(),
        role: "Mid".into(),
        note_type: "matchup".into(),
        title: "vs Zed".into(),
        content: "dodge R".into(),
        difficulty: None,
        created_at: None,
    };
    let id = db::add_champion_note(&db, &t.a, note.clone())
        .await
        .unwrap();
    let mut edit = note;
    edit.id = Some(id.clone());
    edit.title = "hacked".into();
    assert_not_found(
        db::update_champion_note(&db, &t.b, edit.clone()).await,
        "B edits A's champion note",
    );
    assert_not_found(
        db::delete_champion_note(&db, &t.b, &id).await,
        "B deletes A's champion note",
    );
    db::update_champion_note(&db, &t.a, edit).await.unwrap();
    db::delete_champion_note(&db, &t.a, &id).await.unwrap();
}

// ---------------------------------------------------------------------------
// Join requests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn join_request_can_only_be_answered_by_its_team_and_once() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let carol = db::create_user(&db, "carol".into(), "carol@example.com".into(), "h".into())
        .await
        .unwrap();
    db::create_join_request(&db, &carol, &t.team1)
        .await
        .unwrap();
    let req = db::list_pending_join_requests(&db, &t.team1).await.unwrap()[0]
        .id
        .clone();

    // Team 2's leader cannot accept a request addressed to team 1 (it would
    // add Carol to team 2 before M1).
    assert_not_found(
        db::respond_to_join_request(&db, &req, true, &t.team2).await,
        "B answers A's join request",
    );
    assert_eq!(db::get_user_team_id(&db, &carol).await.unwrap(), None);

    // Team 1 accepts once.
    db::respond_to_join_request(&db, &req, true, &t.team1)
        .await
        .unwrap();
    assert_eq!(
        db::get_user_team_id(&db, &carol).await.unwrap(),
        Some(t.team1.clone())
    );

    // Accepting again (no longer pending) fails and adds no second membership.
    assert_not_found(
        db::respond_to_join_request(&db, &req, true, &t.team1).await,
        "re-accepting a handled request",
    );
    let (_, members) = db::get_user_team_with_members(&db, &t.a)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(members.iter().filter(|m| m.user_id == carol).count(), 1);
}

#[tokio::test]
async fn users_belong_to_at_most_one_team() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    // A already leads team 1.
    match db::create_team(&db, &t.a, "Second".into(), "EUW".into()).await {
        Err(DbError::Invalid(_)) => {}
        other => panic!("expected Invalid, got {other:?}"),
    }
    match db::create_join_request(&db, &t.a, &t.team2).await {
        Err(DbError::Invalid(_)) => {}
        other => panic!("expected Invalid, got {other:?}"),
    }
    // A request to a made-up team is rejected.
    let dave = db::create_user(&db, "dave".into(), "dave@example.com".into(), "h".into())
        .await
        .unwrap();
    assert_not_found(
        db::create_join_request(&db, &dave, "team:doesnotexist").await,
        "join request to a missing team",
    );
}

#[tokio::test]
async fn membership_and_leadership_checks() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    assert!(db::is_team_member(&db, &t.a, &t.team1).await.unwrap());
    assert!(!db::is_team_member(&db, &t.a, &t.team2).await.unwrap());
    assert!(db::is_team_leader(&db, &t.a, &t.team1).await.unwrap());
    assert!(!db::is_team_leader(&db, &t.b, &t.team1).await.unwrap());
}

// ---------------------------------------------------------------------------
// Series and fearless lookups
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sand_table_replays_and_boards_are_isolated_between_teams() {
    use lol_team_companion::server::sand_table::import_timeline_str;
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let imported =
        import_timeline_str(include_str!("../fixtures/riot_match_v5_timeline.json")).unwrap();

    let replay = db::store_replay(&db, &t.team1, &imported, None)
        .await
        .unwrap();
    let board = db::create_board(&db, &t.team1, &t.a, "A board".into())
        .await
        .unwrap();
    db::save_board_version(&db, &t.team1, &board, &t.a, serde_json::json!({"v": 1}))
        .await
        .unwrap();

    // B sees none of A's Sand Table data.
    assert!(db::get_replay(&db, &t.team2, &replay)
        .await
        .unwrap()
        .is_none());
    assert!(db::list_replays(&db, &t.team2).await.unwrap().is_empty());
    assert_not_found(
        db::get_replay_frames(&db, &t.team2, &replay).await,
        "replay frames",
    );
    assert_not_found(
        db::get_replay_events(&db, &t.team2, &replay).await,
        "replay events",
    );
    assert_not_found(
        db::delete_replay(&db, &t.team2, &replay).await,
        "delete replay",
    );
    assert!(db::list_boards(&db, &t.team2).await.unwrap().is_empty());
    assert_not_found(
        db::get_board_version(&db, &t.team2, &board, None).await,
        "board version",
    );
    assert_not_found(
        db::save_board_version(&db, &t.team2, &board, &t.b, serde_json::json!({"v": 2})).await,
        "save board version",
    );
    assert_not_found(
        db::delete_board(&db, &t.team2, &board).await,
        "delete board",
    );

    // B importing the same match gets its own copy; A's is untouched.
    let b_replay = db::store_replay(&db, &t.team2, &imported, None)
        .await
        .unwrap();
    assert_ne!(b_replay, replay);
    assert_eq!(
        db::get_replay_frames(&db, &t.team1, &replay)
            .await
            .unwrap()
            .len(),
        4
    );
    let a_board = db::get_board_version(&db, &t.team1, &board, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(a_board.version, 1);
}

#[tokio::test]
async fn series_is_isolated_between_teams() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let series = db::create_series(
        &db,
        &t.team1,
        &t.a,
        "Finals".into(),
        None,
        None,
        "bo3".into(),
        true,
        None,
    )
    .await
    .unwrap();
    assert!(db::get_series(&db, &t.team2, &series)
        .await
        .unwrap()
        .is_none());
    assert_not_found(
        db::delete_series(&db, &t.team2, &series).await,
        "B deletes A's series",
    );
    assert!(
        db::get_fearless_used_champions(&db, &t.team2, &series, None)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(db::get_series(&db, &t.team1, &series)
        .await
        .unwrap()
        .is_some());
    db::delete_series(&db, &t.team1, &series).await.unwrap();
}

// ---------------------------------------------------------------------------
// Cross-tenant: Draft Graph team relations and team games
// ---------------------------------------------------------------------------

#[tokio::test]
async fn team_relation_is_isolated_between_teams() {
    use lol_team_companion::models::graph::{NewRelation, RelationKind};
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let rel = db::create_team_relation(
        &db,
        &t.team1,
        &t.a,
        "alice",
        NewRelation {
            a: "Jinx".into(),
            b: "Thresh".into(),
            kind: RelationKind::Synergy,
            weight: 2,
            reason: "lantern saves Jinx".into(),
        },
    )
    .await
    .unwrap();

    assert!(db::list_team_relations(&db, &t.team2)
        .await
        .unwrap()
        .is_empty());
    assert_not_found(
        db::delete_team_relation(&db, &t.team2, &rel).await,
        "B deletes A's relation",
    );
    let mine = db::list_team_relations(&db, &t.team1).await.unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].author_name, "alice");
    assert_eq!(mine[0].reason, "lantern saves Jinx");
    assert_eq!(mine[0].team_id, t.team1);

    // Shape rules hold on the DB path too (weight, same champion).
    for bad in [
        NewRelation {
            a: "Jinx".into(),
            b: "Jinx".into(),
            kind: RelationKind::Synergy,
            weight: 2,
            reason: "same".into(),
        },
        NewRelation {
            a: "Jinx".into(),
            b: "Lulu".into(),
            kind: RelationKind::Counter,
            weight: 9,
            reason: "weight".into(),
        },
    ] {
        assert!(matches!(
            db::create_team_relation(&db, &t.team1, &t.a, "alice", bad).await,
            Err(DbError::Invalid(_))
        ));
    }
    // Values are bound, not formatted into the query.
    db::create_team_relation(
        &db,
        &t.team1,
        &t.a,
        "alice",
        NewRelation {
            a: "Ahri'); DELETE team_relation; --".into(),
            b: "Zed".into(),
            kind: RelationKind::Counter,
            weight: 1,
            reason: "x'); DELETE user; --".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        db::list_team_relations(&db, &t.team1).await.unwrap().len(),
        2
    );

    db::delete_team_relation(&db, &t.team1, &rel).await.unwrap();
    assert_eq!(
        db::list_team_relations(&db, &t.team1).await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn team_games_are_isolated_between_teams() {
    use lol_team_companion::server::riot::MatchData;
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let m = |id: &str| MatchData {
        match_id: id.into(),
        queue_id: 420,
        game_duration: 1800,
        game_end_epoch_ms: None,
        champion: "Ahri".into(),
        kills: 0,
        deaths: 0,
        assists: 0,
        cs: 0,
        vision_score: 0,
        damage: 0,
        win: true,
    };
    db::store_matches(&db, &t.a, vec![m("EUW1_100"), m("EUW1_101")])
        .await
        .unwrap();
    let mine = db::team_game_rows(&db, &t.team1).await.unwrap();
    assert_eq!(mine.len(), 2);
    assert!(mine.iter().all(|r| r.user_id == t.a));
    assert!(db::team_game_rows(&db, &t.team2).await.unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Injection regression
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_tree_node_treats_ids_as_data_not_query_text() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let tree = db::create_draft_tree(&db, &t.team1, &t.a, "T".into(), None)
        .await
        .unwrap();

    // Before M1 these strings were formatted into the query; the quote broke
    // out of the string literal and the rest ran as SurrealQL.
    let payload = "x'); DELETE user; SELECT * FROM type::record('draft_tree_node', 'x";
    assert_not_found(
        db::create_tree_node(&db, &t.team1, &tree, Some(payload.into()), "evil".into()).await,
        "injected parent id",
    );
    let tree_payload = format!("{tree}'); DELETE user; --");
    assert!(
        db::create_tree_node(&db, &t.team1, &tree_payload, None, "evil".into())
            .await
            .is_err()
    );

    // Nothing was deleted.
    let mut r = db.query("SELECT VALUE id FROM user").await.unwrap();
    let users: Vec<surrealdb::types::RecordId> = r.take(0).unwrap_or_default();
    assert_eq!(users.len(), 2, "users must survive the injection attempt");

    // A legitimate root-level and child node still work.
    let root = db::get_tree_nodes(&db, &t.team1, &tree).await.unwrap()[0]
        .id
        .clone()
        .unwrap();
    db::create_tree_node(&db, &t.team1, &tree, None, "second root".into())
        .await
        .unwrap();
    db::create_tree_node(&db, &t.team1, &tree, Some(root), "child".into())
        .await
        .unwrap();
    let nodes = db::get_tree_nodes(&db, &t.team1, &tree).await.unwrap();
    assert_eq!(nodes.len(), 2);
}

#[tokio::test]
async fn store_matches_binds_values() {
    use lol_team_companion::server::riot::MatchData;
    let db = common::test_db().await;
    let t = tenants(&db).await;
    let m = |id: &str, end: Option<i64>| MatchData {
        match_id: id.into(),
        queue_id: 420,
        game_duration: 1800,
        game_end_epoch_ms: end,
        champion: "Ahri".into(),
        kills: 1,
        deaths: 2,
        assists: 3,
        cs: 200,
        vision_score: 20,
        damage: 15000,
        win: true,
    };
    db::store_matches_with_synced_by(
        &db,
        &t.a,
        vec![
            m("EUW1_1", Some(1_700_000_000_000)),
            m("EUW1_2'; DELETE user; --", None),
        ],
        Some(&t.a),
    )
    .await
    .unwrap();
    db::store_matches(&db, &t.b, vec![m("EUW1_3", None)])
        .await
        .unwrap();
    let rows = db::get_team_match_stats(&db, &t.team1).await.unwrap();
    assert_eq!(rows.len(), 2);
    let mut r = db.query("SELECT VALUE id FROM user").await.unwrap();
    let users: Vec<surrealdb::types::RecordId> = r.take(0).unwrap_or_default();
    assert_eq!(users.len(), 2);
}

// ---------------------------------------------------------------------------
// Registration rules and invites
// ---------------------------------------------------------------------------

#[tokio::test]
async fn duplicate_email_and_username_are_rejected_case_insensitively() {
    let db = common::test_db().await;
    db::create_user(
        &db,
        "Alice".into(),
        " Alice@Example.COM ".into(),
        "h".into(),
    )
    .await
    .unwrap();
    // Stored normalised.
    let mut r = db.query("SELECT VALUE email FROM user").await.unwrap();
    let emails: Vec<String> = r.take(0).unwrap();
    assert_eq!(emails, vec!["alice@example.com".to_string()]);

    match db::create_user(&db, "other".into(), "ALICE@example.com".into(), "h".into()).await {
        Err(DbError::Invalid(msg)) => assert_eq!(msg, db::GENERIC_REGISTRATION_ERROR),
        other => panic!("expected generic Invalid, got {other:?}"),
    }
    match db::create_user(&db, "aLiCe".into(), "new@example.com".into(), "h".into()).await {
        Err(DbError::Invalid(msg)) => assert!(msg.contains("username"), "{msg}"),
        other => panic!("expected username Invalid, got {other:?}"),
    }
    let bob = db::create_user(&db, "bob".into(), "bob@example.com".into(), "h".into())
        .await
        .unwrap();
    match db::update_username(&db, &bob, "ALICE".into()).await {
        Err(DbError::Invalid(_)) => {}
        other => panic!("expected Invalid, got {other:?}"),
    }
    db::update_username(&db, &bob, "Bobby".into())
        .await
        .unwrap();
}

#[tokio::test]
async fn valid_invite_is_consumed_until_used_up() {
    let db = common::test_db().await;
    db::create_invite(&db, None, "open-1234".into(), 2, None, None)
        .await
        .unwrap();
    // Codes are case-insensitive (stored upper-case).
    assert!(db::consume_invite(&db, "OPEN-1234").await.unwrap());
    assert!(db::consume_invite(&db, " open-1234 ").await.unwrap());
    assert!(
        !db::consume_invite(&db, "open-1234").await.unwrap(),
        "third use must fail (max_uses = 2)"
    );
    // A released use can be taken again.
    db::release_invite(&db, "open-1234").await.unwrap();
    assert!(db::consume_invite(&db, "open-1234").await.unwrap());
    let invites = db::list_invites(&db).await.unwrap();
    assert_eq!(invites[0].uses, 2);
}

#[tokio::test]
async fn unknown_expired_and_disabled_invites_are_rejected() {
    let db = common::test_db().await;
    assert!(!db::consume_invite(&db, "NOPE").await.unwrap());
    assert!(!db::consume_invite(&db, "").await.unwrap());

    let past = chrono::Utc::now() - chrono::Duration::days(1);
    db::create_invite(&db, None, "EXPIRED".into(), 5, Some(past), None)
        .await
        .unwrap();
    assert!(!db::consume_invite(&db, "EXPIRED").await.unwrap());

    let future = chrono::Utc::now() + chrono::Duration::days(1);
    let id = db::create_invite(&db, None, "SOON".into(), 5, Some(future), None)
        .await
        .unwrap();
    assert!(db::consume_invite(&db, "SOON").await.unwrap());
    db::disable_invite(&db, &id).await.unwrap();
    assert!(!db::consume_invite(&db, "SOON").await.unwrap());

    // Duplicate codes and zero max uses are refused.
    assert!(matches!(
        db::create_invite(&db, None, "soon".into(), 1, None, None).await,
        Err(DbError::Invalid(_))
    ));
    assert!(matches!(
        db::create_invite(&db, None, "ZERO".into(), 0, None, None).await,
        Err(DbError::Invalid(_))
    ));
    assert_not_found(
        db::disable_invite(&db, "invite:missing").await,
        "disable unknown invite",
    );
}

#[tokio::test]
async fn ensure_invite_is_idempotent_and_codes_are_random() {
    let db = common::test_db().await;
    db::ensure_invite(&db, "boot", 3).await.unwrap();
    db::ensure_invite(&db, "BOOT", 99).await.unwrap();
    let invites = db::list_invites(&db).await.unwrap();
    assert_eq!(invites.len(), 1);
    assert_eq!(invites[0].max_uses, 3, "existing code is left unchanged");

    let a = db::generate_invite_code();
    let b = db::generate_invite_code();
    assert_ne!(a, b);
    assert_eq!(a.len(), 14);
    assert_eq!(a.matches('-').count(), 2);
}

#[test]
fn invite_gate_can_only_be_disabled_in_dev() {
    use lol_team_companion::server::config::closed_beta_enabled;
    assert!(closed_beta_enabled(false, Some("false")), "prod ignores it");
    assert!(closed_beta_enabled(true, None), "dev defaults to on");
    assert!(
        !closed_beta_enabled(true, Some("false")),
        "dev can turn it off"
    );
}

#[tokio::test]
async fn admin_flag_round_trip() {
    let db = common::test_db().await;
    let t = tenants(&db).await;
    db::set_user_admin(&db, &t.a, true).await.unwrap();
    let mut r = db
        .query("SELECT VALUE is_admin FROM type::record('user', $k)")
        .bind(("k", db::record_key(&t.a, "user")))
        .await
        .unwrap();
    let flags: Vec<Option<bool>> = r.take(0).unwrap();
    assert_eq!(flags, vec![Some(true)]);
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn expired_sessions_are_not_loaded_and_are_purged() {
    use lol_team_companion::server::session_store::SurrealSessionStore;
    use tower_sessions::cookie::time::{Duration, OffsetDateTime};
    use tower_sessions_core::session::{Id, Record};
    use tower_sessions_core::{ExpiredDeletion, SessionStore};

    let db = common::test_db().await;
    let store = SurrealSessionStore::new(db.clone(), "sessions".into());

    let mut live = Record {
        id: Id::default(),
        data: Default::default(),
        expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
    };
    let mut dead = Record {
        id: Id::default(),
        data: Default::default(),
        expiry_date: OffsetDateTime::now_utc() - Duration::minutes(1),
    };
    store.create(&mut live).await.unwrap();
    store.create(&mut dead).await.unwrap();

    assert!(store.load(&live.id).await.unwrap().is_some());
    assert!(
        store.load(&dead.id).await.unwrap().is_none(),
        "expired session must load as absent"
    );

    // A row from before M1 (no expires_at) and another expired row.
    let mut old = Record {
        id: Id::default(),
        data: Default::default(),
        expiry_date: OffsetDateTime::now_utc() - Duration::days(2),
    };
    store.create(&mut old).await.unwrap();
    db.query("CREATE sessions:legacy SET data = 'legacy'")
        .await
        .unwrap()
        .check()
        .unwrap();

    store.delete_expired().await.unwrap();
    let mut r = db.query("SELECT VALUE id FROM sessions").await.unwrap();
    let ids: Vec<surrealdb::types::RecordId> = r.take(0).unwrap();
    assert_eq!(ids.len(), 1, "only the live session remains: {ids:?}");
    assert!(store.load(&live.id).await.unwrap().is_some());

    // Saving extends the expiry.
    live.expiry_date = OffsetDateTime::now_utc() + Duration::days(30);
    store.save(&live).await.unwrap();
    let loaded = store.load(&live.id).await.unwrap().unwrap();
    assert_eq!(
        loaded.expiry_date.unix_timestamp(),
        live.expiry_date.unix_timestamp()
    );
}

// ---------------------------------------------------------------------------
// Concurrency (review follow-up)
// ---------------------------------------------------------------------------

/// SurrealKV sometimes hangs while shutting a datastore down at runtime
/// teardown (a join on its commit thread, seen with gdb), so the SurrealKV
/// tests run on a runtime that is never dropped and never drop the datastore.
fn on_kv_runtime<F: std::future::Future>(f: F) -> F::Output {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap()
    })
    .block_on(f)
}

/// A SurrealKV-backed DB (the production engine) in a fresh temp directory,
/// plus that directory (remove it when done).
async fn surrealkv_db() -> (std::sync::Arc<Surreal<Db>>, std::path::PathBuf) {
    use surrealdb::engine::local::SurrealKv;
    let dir = std::env::temp_dir().join(format!(
        "ltc-review-{}-{}",
        std::process::id(),
        db::generate_invite_code()
    ));
    let db = Surreal::new::<SurrealKv>(dir.to_string_lossy().to_string())
        .await
        .unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    // Baseline plus migrations, so the unique indexes added by migrations
    // (username_lc, team_member.user) exist as in production.
    db::prepare(&db).await.unwrap();
    (std::sync::Arc::new(db), dir)
}

async fn last_invite_use_is_taken_once(db: std::sync::Arc<Surreal<Db>>) {
    db::create_invite(&db, None, "LAST-ONE".into(), 1, None, None)
        .await
        .unwrap();
    let mut handles = Vec::new();
    for _ in 0..16 {
        let db = db.clone();
        handles.push(tokio::spawn(async move {
            db::consume_invite(&db, "LAST-ONE").await
        }));
    }
    let mut ok = 0;
    for h in handles {
        // An error (e.g. a write conflict) is acceptable; a second success is not.
        if let Ok(true) = h.await.unwrap() {
            ok += 1;
        }
    }
    let invites = db::list_invites(&db).await.unwrap();
    assert_eq!(
        (ok, invites[0].uses),
        (1, 1),
        "exactly one registration may take the last use"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_registrations_cannot_overuse_an_invite_mem() {
    last_invite_use_is_taken_once(common::test_db().await).await;
}

#[test]
fn concurrent_registrations_cannot_overuse_an_invite_surrealkv() {
    on_kv_runtime(async {
        let (db, dir) = surrealkv_db().await;
        last_invite_use_is_taken_once(db.clone()).await;
        std::mem::forget(db);
        let _ = std::fs::remove_dir_all(dir);
    });
}

async fn double_accept_adds_one_membership(db: std::sync::Arc<Surreal<Db>>) {
    let t = tenants(&db).await;
    let carol = db::create_user(&db, "carol".into(), "carol@example.com".into(), "h".into())
        .await
        .unwrap();
    db::create_join_request(&db, &carol, &t.team1)
        .await
        .unwrap();
    let req = db::list_pending_join_requests(&db, &t.team1).await.unwrap()[0]
        .id
        .clone();
    let mut handles = Vec::new();
    for _ in 0..8 {
        let (db, req, team) = (db.clone(), req.clone(), t.team1.clone());
        handles.push(tokio::spawn(async move {
            db::respond_to_join_request(&db, &req, true, &team).await
        }));
    }
    let mut ok = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            ok += 1;
        }
    }
    assert_eq!(ok, 1, "only one accept may succeed");
    let mut r = db
        .query("SELECT VALUE id FROM team_member WHERE user = type::record('user', $k)")
        .bind(("k", db::record_key(&carol, "user")))
        .await
        .unwrap();
    let rows: Vec<surrealdb::types::RecordId> = r.take(0).unwrap();
    assert_eq!(
        rows.len(),
        1,
        "a double-clicked accept must not add two memberships"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_accepts_add_one_membership_mem() {
    double_accept_adds_one_membership(common::test_db().await).await;
}

#[test]
fn concurrent_accepts_add_one_membership_surrealkv() {
    on_kv_runtime(async {
        let (db, dir) = surrealkv_db().await;
        double_accept_adds_one_membership(db.clone()).await;
        std::mem::forget(db);
        let _ = std::fs::remove_dir_all(dir);
    });
}

async fn cross_team_accepts_add_one_membership(db: std::sync::Arc<Surreal<Db>>) {
    let t = tenants(&db).await;
    let carol = db::create_user(&db, "carol".into(), "carol@example.com".into(), "h".into())
        .await
        .unwrap();
    db::create_join_request(&db, &carol, &t.team1)
        .await
        .unwrap();
    db::create_join_request(&db, &carol, &t.team2)
        .await
        .unwrap();
    let r1 = db::list_pending_join_requests(&db, &t.team1).await.unwrap()[0]
        .id
        .clone();
    let r2 = db::list_pending_join_requests(&db, &t.team2).await.unwrap()[0]
        .id
        .clone();
    // Both leaders accept at the same moment, while Carol also creates a team.
    let (d1, d2, d3) = (db.clone(), db.clone(), db.clone());
    let (t1, t2, c) = (t.team1.clone(), t.team2.clone(), carol.clone());
    let a = tokio::spawn(async move { db::respond_to_join_request(&d1, &r1, true, &t1).await });
    let b = tokio::spawn(async move { db::respond_to_join_request(&d2, &r2, true, &t2).await });
    let own =
        tokio::spawn(async move { db::create_team(&d3, &c, "Carol's".into(), "EUW".into()).await });
    let ok = [a.await.unwrap().is_ok(), b.await.unwrap().is_ok()]
        .into_iter()
        .chain([own.await.unwrap().is_ok()])
        .filter(|ok| *ok)
        .count();
    let mut r = db
        .query("SELECT VALUE id FROM team_member WHERE user = type::record('user', $k)")
        .bind(("k", db::record_key(&carol, "user")))
        .await
        .unwrap();
    let rows: Vec<surrealdb::types::RecordId> = r.take(0).unwrap();
    assert_eq!(
        (ok, rows.len()),
        (1, 1),
        "a user ends up in exactly one team"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_cross_team_accepts_keep_one_team_mem() {
    cross_team_accepts_add_one_membership(common::test_db().await).await;
}

#[test]
fn concurrent_cross_team_accepts_keep_one_team_surrealkv() {
    on_kv_runtime(async {
        let (db, dir) = surrealkv_db().await;
        cross_team_accepts_add_one_membership(db.clone()).await;
        std::mem::forget(db);
        let _ = std::fs::remove_dir_all(dir);
    });
}

// ---------------------------------------------------------------------------
// Unique indexes under concurrency (M6 review). M6 dropped the check-first
// queries for usernames and emails and relies on the unique indexes alone;
// board versions and `join_team` have no process lock either.
// ---------------------------------------------------------------------------

async fn concurrent_signups_take_a_name_once(db: std::sync::Arc<Surreal<Db>>) {
    let mut handles = Vec::new();
    for i in 0..12 {
        let db = db.clone();
        // Case variants of one username, each with its own email.
        let name = ["Racer", "racer", "RACER"][i % 3].to_string();
        handles.push(tokio::spawn(async move {
            db::create_user(&db, name, format!("racer{i}@example.com"), "h".into()).await
        }));
    }
    let mut ok = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            ok += 1;
        }
    }
    let mut handles = Vec::new();
    for i in 0..12 {
        let db = db.clone();
        let email = ["same@example.com", "Same@Example.com"][i % 2].to_string();
        handles.push(tokio::spawn(async move {
            db::create_user(&db, format!("same{i}"), email, "h".into()).await
        }));
    }
    let mut ok_email = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            ok_email += 1;
        }
    }
    let mut r = db
        .query(
            "RETURN [array::len((SELECT VALUE id FROM user WHERE username_lc = 'racer')), \
                     array::len((SELECT VALUE id FROM user WHERE email = 'same@example.com'))]",
        )
        .await
        .unwrap();
    let rows: Vec<i64> = r.take(0).unwrap();
    assert_eq!(
        (ok, ok_email, rows),
        (1, 1, vec![1, 1]),
        "one account per username and per email"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_signups_take_a_name_once_mem() {
    concurrent_signups_take_a_name_once(common::test_db().await).await;
}

#[test]
fn concurrent_signups_take_a_name_once_surrealkv() {
    on_kv_runtime(async {
        let (db, dir) = surrealkv_db().await;
        concurrent_signups_take_a_name_once(db.clone()).await;
        std::mem::forget(db);
        let _ = std::fs::remove_dir_all(dir);
    });
}

async fn concurrent_board_saves_get_distinct_versions(db: std::sync::Arc<Surreal<Db>>) {
    let t = tenants(&db).await;
    let board = db::create_board(&db, &t.team1, &t.a, "Race".into())
        .await
        .unwrap();
    let mut handles = Vec::new();
    for i in 0..8 {
        let (db, board, team, user) = (db.clone(), board.clone(), t.team1.clone(), t.a.clone());
        handles.push(tokio::spawn(async move {
            db::save_board_version(&db, &team, &board, &user, serde_json::json!({ "i": i })).await
        }));
    }
    let mut versions = Vec::new();
    for h in handles {
        if let Ok(v) = h.await.unwrap() {
            versions.push(v);
        }
    }
    let mut r = db
        .query("SELECT VALUE version FROM board_version WHERE board = type::record('board', $k)")
        .bind(("k", db::record_key(&board, "board")))
        .await
        .unwrap();
    let mut stored: Vec<i64> = r.take(0).unwrap();
    stored.sort();
    versions.sort();
    assert!(!versions.is_empty());
    assert_eq!(stored, versions, "every reported version is stored once");
    let mut dedup = stored.clone();
    dedup.dedup();
    assert_eq!(dedup, stored, "no two saves share a version number");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_board_saves_get_distinct_versions_mem() {
    concurrent_board_saves_get_distinct_versions(common::test_db().await).await;
}

#[test]
fn concurrent_board_saves_get_distinct_versions_surrealkv() {
    on_kv_runtime(async {
        let (db, dir) = surrealkv_db().await;
        concurrent_board_saves_get_distinct_versions(db.clone()).await;
        std::mem::forget(db);
        let _ = std::fs::remove_dir_all(dir);
    });
}

async fn concurrent_joins_keep_one_team(db: std::sync::Arc<Surreal<Db>>) {
    let t = tenants(&db).await;
    let dave = db::create_user(&db, "dave".into(), "dave@example.com".into(), "h".into())
        .await
        .unwrap();
    let mut handles = Vec::new();
    for i in 0..8 {
        let (db, dave) = (db.clone(), dave.clone());
        let team = if i % 2 == 0 {
            t.team1.clone()
        } else {
            t.team2.clone()
        };
        handles.push(tokio::spawn(async move {
            db::join_team(&db, &dave, &team).await
        }));
    }
    for h in handles {
        let _ = h.await.unwrap();
    }
    let mut r = db
        .query("SELECT VALUE id FROM team_member WHERE user = type::record('user', $k)")
        .bind(("k", db::record_key(&dave, "user")))
        .await
        .unwrap();
    let rows: Vec<surrealdb::types::RecordId> = r.take(0).unwrap();
    assert_eq!(rows.len(), 1, "the unique index keeps one membership");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_joins_keep_one_team_mem() {
    concurrent_joins_keep_one_team(common::test_db().await).await;
}

#[test]
fn concurrent_joins_keep_one_team_surrealkv() {
    on_kv_runtime(async {
        let (db, dir) = surrealkv_db().await;
        concurrent_joins_keep_one_team(db.clone()).await;
        std::mem::forget(db);
        let _ = std::fs::remove_dir_all(dir);
    });
}
