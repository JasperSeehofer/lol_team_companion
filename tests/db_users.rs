#![cfg(feature = "ssr")]

mod common;

use lol_team_companion::server::db;

#[tokio::test]
async fn test_create_user_success() {
    let db = common::test_db().await;
    let id = db::create_user(
        &db,
        "alice".into(),
        "alice@example.com".into(),
        "hash1".into(),
    )
    .await
    .unwrap();
    assert!(
        id.starts_with("user:"),
        "id should have user: prefix, got {id}"
    );
}

#[tokio::test]
async fn test_create_user_duplicate_email_fails() {
    let db = common::test_db().await;
    db::create_user(
        &db,
        "alice".into(),
        "dup@example.com".into(),
        "hash1".into(),
    )
    .await
    .unwrap();
    let result = db::create_user(
        &db,
        "alice2".into(),
        "dup@example.com".into(),
        "hash2".into(),
    )
    .await;
    assert!(result.is_err(), "duplicate email should fail");
}

#[tokio::test]
async fn test_get_nonexistent_user_returns_none() {
    let db = common::test_db().await;
    // AuthBackend::get_user requires auth setup; test via get_user_team_id returning None
    let result = db::get_user_team_id(&db, "user:nonexistent").await.unwrap();
    assert!(result.is_none());
}
