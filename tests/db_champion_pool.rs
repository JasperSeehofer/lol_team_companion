#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

async fn make_user(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> String {
    db::create_user(
        db,
        "pooluser".into(),
        "pool@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn test_add_champion_to_pool() {
    let db = common::test_db().await;
    let user_id = make_user(&db).await;

    db::add_to_champion_pool(&db, &user_id, "Jinx".into(), "bot".into())
        .await
        .unwrap();

    let pool = db::get_champion_pool(&db, &user_id).await.unwrap();
    assert_eq!(pool.len(), 1);
    assert_eq!(pool[0].champion, "Jinx");
    assert_eq!(pool[0].role, "bot");
    assert_eq!(pool[0].tier, "comfort"); // schema DEFAULT 'comfort'
}

#[tokio::test]
async fn test_add_champion_duplicate_is_idempotent() {
    let db = common::test_db().await;
    let user_id = make_user(&db).await;

    db::add_to_champion_pool(&db, &user_id, "Azir".into(), "mid".into())
        .await
        .unwrap();
    // Adding again should not fail and should not create a duplicate
    db::add_to_champion_pool(&db, &user_id, "Azir".into(), "mid".into())
        .await
        .unwrap();

    let pool = db::get_champion_pool(&db, &user_id).await.unwrap();
    assert_eq!(pool.len(), 1, "duplicate add should not create extra entry");
}

#[tokio::test]
async fn test_update_champion_tier() {
    let db = common::test_db().await;
    let user_id = make_user(&db).await;

    db::add_to_champion_pool(&db, &user_id, "Caitlyn".into(), "bot".into())
        .await
        .unwrap();
    db::update_champion_tier(&db, &user_id, "Caitlyn", "bot", "s".into())
        .await
        .unwrap();

    let pool = db::get_champion_pool(&db, &user_id).await.unwrap();
    assert_eq!(pool[0].tier, "s");
}

#[tokio::test]
async fn test_remove_champion_from_pool() {
    let db = common::test_db().await;
    let user_id = make_user(&db).await;

    db::add_to_champion_pool(&db, &user_id, "Thresh".into(), "support".into())
        .await
        .unwrap();
    db::remove_from_champion_pool(&db, &user_id, "Thresh".into(), "support".into())
        .await
        .unwrap();

    let pool = db::get_champion_pool(&db, &user_id).await.unwrap();
    assert!(pool.is_empty(), "pool should be empty after removal");
}

#[tokio::test]
async fn test_get_pool_returns_all_entries() {
    let db = common::test_db().await;
    let user_id = make_user(&db).await;

    for (champ, role) in [("Jinx", "bot"), ("Azir", "mid"), ("Thresh", "support")] {
        db::add_to_champion_pool(&db, &user_id, champ.into(), role.into())
            .await
            .unwrap();
    }

    let pool = db::get_champion_pool(&db, &user_id).await.unwrap();
    assert_eq!(pool.len(), 3);
}
