#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::models::draft::DraftAction;
use lol_team_companion::models::game_plan::GamePlan;
use lol_team_companion::server::db;

async fn setup(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> (String, String) {
    let user_id = db::create_user(
        db,
        "pipeline_user".into(),
        "pipeline@example.com".into(),
        "h".into(),
    )
    .await
    .unwrap();
    let team_id = db::create_team(db, &user_id, "PipelineTeam".into(), "EUW".into())
        .await
        .unwrap();
    (user_id, team_id)
}

fn sample_actions(draft_id: &str) -> Vec<DraftAction> {
    vec![
        DraftAction {
            id: None,
            draft_id: draft_id.into(),
            phase: "ban1".into(),
            side: "blue".into(),
            champion: "Azir".into(),
            order: 0,
            comment: None,
            role: None,
        },
        DraftAction {
            id: None,
            draft_id: draft_id.into(),
            phase: "pick1".into(),
            side: "blue".into(),
            champion: "Jinx".into(),
            order: 1,
            comment: Some("strong ADC".into()),
            role: None,
        },
    ]
}

#[tokio::test]
async fn test_get_draft_for_prefill_found() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;
    let actions = sample_actions("placeholder");

    let draft_id = db::save_draft(
        &db,
        &team_id,
        &user_id,
        "PrefillDraft".into(),
        Some("Rivals".into()),
        None,
        vec![],
        actions,
        None,
        "blue".into(),
        vec![],
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let result = db::get_draft_for_prefill(&db, &draft_id).await.unwrap();
    assert!(result.is_some(), "Expected Some(draft) for valid draft_id");
    let draft = result.unwrap();
    assert_eq!(draft.name, "PrefillDraft");
    assert_eq!(draft.our_side, "blue");
    assert_eq!(draft.actions.len(), 2, "Expected 2 actions");
}

#[tokio::test]
async fn test_get_draft_for_prefill_not_found() {
    let db = common::test_db().await;

    let result = db::get_draft_for_prefill(&db, "draft:nonexistent").await.unwrap();
    assert!(result.is_none(), "Expected None for nonexistent draft_id");
}

#[tokio::test]
async fn test_get_game_plans_for_draft_found() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    // Create a draft first to get a real draft ID
    let draft_id = db::save_draft(
        &db,
        &team_id,
        &user_id,
        "LinkedDraft".into(),
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
    .unwrap();

    // Create a game plan linked to that draft
    let plan = GamePlan {
        id: None,
        team_id: team_id.clone(),
        draft_id: Some(draft_id.clone()),
        name: "Plan for Draft".into(),
        our_champions: vec![],
        enemy_champions: vec![],
        win_conditions: vec![],
        objective_priority: vec![],
        teamfight_strategy: "Teamfight".into(),
        early_game: None,
        top_strategy: None,
        jungle_strategy: None,
        mid_strategy: None,
        bot_strategy: None,
        support_strategy: None,
        notes: None,
        win_condition_tag: None,
    };
    db::save_game_plan(&db, plan, &user_id).await.unwrap();

    let plans = db::get_game_plans_for_draft(&db, &draft_id).await.unwrap();
    assert_eq!(plans.len(), 1, "Expected 1 game plan linked to draft");
    assert_eq!(plans[0].draft_id, Some(draft_id));
}

#[tokio::test]
async fn test_get_game_plans_for_draft_empty() {
    let db = common::test_db().await;
    let (user_id, team_id) = setup(&db).await;

    // Create a draft that no game plan references
    let draft_id = db::save_draft(
        &db,
        &team_id,
        &user_id,
        "UnlinkedDraft".into(),
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
    .unwrap();

    let plans = db::get_game_plans_for_draft(&db, &draft_id).await.unwrap();
    assert!(plans.is_empty(), "Expected empty vec for draft with no game plans");
}
