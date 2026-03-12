#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::models::draft::DraftAction;
use lol_team_companion::server::db;

async fn setup(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> (String, String, String) {
    let user_id = db::create_user(
        db,
        "drafter".into(),
        "drafter@example.com".into(),
        "h".into(),
    )
    .await
    .unwrap();
    let team_id = db::create_team(db, &user_id, "DraftTeam".into(), "EUW".into())
        .await
        .unwrap();
    (user_id, team_id, String::new())
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
        },
        DraftAction {
            id: None,
            draft_id: draft_id.into(),
            phase: "pick1".into(),
            side: "blue".into(),
            champion: "Jinx".into(),
            order: 1,
        },
    ]
}

#[tokio::test]
async fn test_save_draft_with_actions() {
    let db = common::test_db().await;
    let (user_id, team_id, _) = setup(&db).await;
    let actions = sample_actions("placeholder");

    let draft_id = db::save_draft(
        &db,
        &team_id,
        &user_id,
        "Draft 1".into(),
        Some("Opponents".into()),
        None,
        vec![],
        actions,
        None,
        "blue".into(),
    )
    .await
    .unwrap();

    assert!(draft_id.starts_with("draft:"));

    let drafts = db::list_drafts(&db, &team_id).await.unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].name, "Draft 1");
    assert_eq!(drafts[0].actions.len(), 2);
}

#[tokio::test]
async fn test_update_draft_replaces_actions() {
    let db = common::test_db().await;
    let (user_id, team_id, _) = setup(&db).await;

    let draft_id = db::save_draft(
        &db,
        &team_id,
        &user_id,
        "Original".into(),
        None,
        None,
        vec![],
        sample_actions(""),
        None,
        "blue".into(),
    )
    .await
    .unwrap();

    // Update with only one action
    let new_actions = vec![DraftAction {
        id: None,
        draft_id: draft_id.clone(),
        phase: "ban1".into(),
        side: "red".into(),
        champion: "Thresh".into(),
        order: 0,
    }];

    db::update_draft(
        &db,
        &draft_id,
        "Updated".into(),
        None,
        Some("notes".into()),
        vec!["comment".into()],
        new_actions,
        Some("A".into()),
        "red".into(),
    )
    .await
    .unwrap();

    let drafts = db::list_drafts(&db, &team_id).await.unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].name, "Updated");
    assert_eq!(drafts[0].our_side, "red");
    assert_eq!(drafts[0].actions.len(), 1, "old actions should be replaced");
    assert_eq!(drafts[0].actions[0].champion, "Thresh");
}

#[tokio::test]
async fn test_list_drafts_aggregates_actions_correctly() {
    let db = common::test_db().await;
    let (user_id, team_id, _) = setup(&db).await;

    db::save_draft(
        &db,
        &team_id,
        &user_id,
        "D1".into(),
        None,
        None,
        vec![],
        sample_actions(""),
        None,
        "blue".into(),
    )
    .await
    .unwrap();

    db::save_draft(
        &db,
        &team_id,
        &user_id,
        "D2".into(),
        None,
        None,
        vec![],
        vec![],
        None,
        "red".into(),
    )
    .await
    .unwrap();

    let drafts = db::list_drafts(&db, &team_id).await.unwrap();
    assert_eq!(drafts.len(), 2);
    let d1 = drafts.iter().find(|d| d.name == "D1").unwrap();
    let d2 = drafts.iter().find(|d| d.name == "D2").unwrap();
    assert_eq!(d1.actions.len(), 2);
    assert_eq!(d2.actions.len(), 0);
}

#[tokio::test]
async fn test_delete_draft() {
    let db = common::test_db().await;
    let (user_id, team_id, _) = setup(&db).await;

    let draft_id = db::save_draft(
        &db,
        &team_id,
        &user_id,
        "ToDelete".into(),
        None,
        None,
        vec![],
        sample_actions(""),
        None,
        "blue".into(),
    )
    .await
    .unwrap();

    db::delete_draft(&db, &draft_id).await.unwrap();

    let drafts = db::list_drafts(&db, &team_id).await.unwrap();
    assert!(drafts.is_empty(), "draft should be deleted");
}
