use crate::common;

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
            role_guessed: false,
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
            role_guessed: false,
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

    let result = db::get_draft_for_prefill(&db, &team_id, &draft_id)
        .await
        .unwrap();
    assert!(result.is_some(), "Expected Some(draft) for valid draft_id");
    let draft = result.unwrap();
    assert_eq!(draft.name, "PrefillDraft");
    assert_eq!(draft.our_side, "blue");
    assert_eq!(draft.actions.len(), 2, "Expected 2 actions");
}

#[tokio::test]
async fn test_get_draft_for_prefill_not_found() {
    let db = common::test_db().await;

    let result = db::get_draft_for_prefill(&db, "team:nonexistent", "draft:nonexistent")
        .await
        .unwrap();
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

    let plans = db::get_game_plans_for_draft(&db, &team_id, &draft_id)
        .await
        .unwrap();
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

    let plans = db::get_game_plans_for_draft(&db, &team_id, &draft_id)
        .await
        .unwrap();
    assert!(
        plans.is_empty(),
        "Expected empty vec for draft with no game plans"
    );
}

/// M6: drafts and plans link by record, so the dashboard's "drafts without a
/// plan" and "plans without a review" counts are right (the first used to
/// compare a field the draft table does not have, the second compared a
/// record with a string, so both counted everything).
#[tokio::test]
async fn dashboard_counts_follow_record_links() {
    use lol_team_companion::models::game_plan::{GamePlan, PostGameLearning};
    let d = common::test_db().await;
    let u = db::create_user(&d, "lead".into(), "lead@example.com".into(), "h".into())
        .await
        .unwrap();
    let t = db::create_team(&d, &u, "Links".into(), "EUW".into())
        .await
        .unwrap();
    let mk_draft = |name: &'static str| {
        let (d, t, u) = (&d, t.clone(), u.clone());
        async move {
            db::save_draft(
                d,
                &t,
                &u,
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
    };
    let d1 = mk_draft("planned").await;
    let _d2 = mk_draft("unplanned").await;
    let plan = GamePlan {
        id: None,
        team_id: t.clone(),
        draft_id: Some(d1.clone()),
        name: "P".into(),
        our_champions: vec![],
        enemy_champions: vec![],
        win_conditions: vec![],
        objective_priority: vec![],
        teamfight_strategy: "x".into(),
        early_game: None,
        top_strategy: None,
        jungle_strategy: None,
        mid_strategy: None,
        bot_strategy: None,
        support_strategy: None,
        notes: None,
        win_condition_tag: None,
    };
    let p1 = db::save_game_plan(&d, plan.clone(), &u).await.unwrap();
    let _p2 = db::save_game_plan(
        &d,
        GamePlan {
            draft_id: None,
            ..plan
        },
        &u,
    )
    .await
    .unwrap();
    db::save_post_game_learning(
        &d,
        PostGameLearning {
            id: None,
            team_id: t.clone(),
            match_riot_id: None,
            game_plan_id: Some(p1.clone()),
            draft_id: Some(d1.clone()),
            what_went_well: vec![],
            improvements: vec![],
            action_items: vec![],
            open_notes: None,
            created_by: u.clone(),
            win_loss: Some("win".into()),
            rating: Some(4),
        },
    )
    .await
    .unwrap();

    let summary = db::get_dashboard_summary(&d, &t).await.unwrap();
    assert_eq!(summary.drafts_without_game_plan, 1);
    assert_eq!(summary.game_plans_without_post_game, 1);
    let linked = db::get_game_plans_for_draft(&d, &t, &d1).await.unwrap();
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].draft_id.as_deref(), Some(d1.as_str()));
}
