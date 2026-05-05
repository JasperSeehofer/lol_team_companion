#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(d: &surrealdb::Surreal<surrealdb::engine::local::Db>, name: &str) -> String {
    db::create_user(
        d,
        name.into(),
        format!("{name}@example.com"),
        "hash".into(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn upsert_creates_new_goal() {
    let d = common::test_db().await;
    let u = make_user(&d, "alice").await;
    db::upsert_personal_goal(&d, &u, "rank_target", "DIAMOND:IV")
        .await
        .unwrap();
    let goals = db::get_personal_goals(&d, &u).await.unwrap();
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0].goal_type, "rank_target");
    assert_eq!(goals[0].target_value, "DIAMOND:IV");
}

#[tokio::test]
async fn upsert_overwrites_existing_goal() {
    let d = common::test_db().await;
    let u = make_user(&d, "bob").await;
    db::upsert_personal_goal(&d, &u, "rank_target", "GOLD:I")
        .await
        .unwrap();
    db::upsert_personal_goal(&d, &u, "rank_target", "MASTER:")
        .await
        .unwrap();
    let goals = db::get_personal_goals(&d, &u).await.unwrap();
    assert_eq!(goals.len(), 1, "expected exactly one goal after overwrite");
    assert_eq!(goals[0].target_value, "MASTER:");
}

#[tokio::test]
async fn upsert_different_types_creates_separate_goals() {
    let d = common::test_db().await;
    let u = make_user(&d, "carol").await;
    db::upsert_personal_goal(&d, &u, "rank_target", "DIAMOND:IV")
        .await
        .unwrap();
    db::upsert_personal_goal(&d, &u, "cs_per_min", "7.5")
        .await
        .unwrap();
    db::upsert_personal_goal(&d, &u, "deaths_per_game", "4")
        .await
        .unwrap();
    let goals = db::get_personal_goals(&d, &u).await.unwrap();
    assert_eq!(goals.len(), 3);
}

#[tokio::test]
async fn cross_user_goal_isolation() {
    let d = common::test_db().await;
    let a = make_user(&d, "userA").await;
    let b = make_user(&d, "userB").await;
    db::upsert_personal_goal(&d, &a, "cs_per_min", "8.0")
        .await
        .unwrap();
    let a_goals = db::get_personal_goals(&d, &a).await.unwrap();
    let b_goals = db::get_personal_goals(&d, &b).await.unwrap();
    assert_eq!(a_goals.len(), 1);
    assert_eq!(b_goals.len(), 0, "userB must not see userA's goals");
}

#[tokio::test]
async fn get_personal_goals_empty_returns_empty_vec() {
    let d = common::test_db().await;
    let u = make_user(&d, "fresh").await;
    let goals = db::get_personal_goals(&d, &u).await.unwrap();
    assert!(goals.is_empty());
}
