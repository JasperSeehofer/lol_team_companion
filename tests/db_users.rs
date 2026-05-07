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

// ---------------------------------------------------------------------------
// Theme persistence tests (Phase 17 plan 17-01)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_user_theme_persists_demacia() {
    let db = common::test_db().await;
    let user_id = db::create_user(
        &db,
        "theme_user1".into(),
        "theme1@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap();
    db::set_user_theme(&db, &user_id, "demacia").await.unwrap();
    let theme = db::get_user_theme(&db, &user_id).await.unwrap();
    assert_eq!(theme, "demacia");
}

#[tokio::test]
async fn test_set_user_theme_persists_pandemonium() {
    let db = common::test_db().await;
    let user_id = db::create_user(
        &db,
        "theme_user2".into(),
        "theme2@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap();
    db::set_user_theme(&db, &user_id, "pandemonium")
        .await
        .unwrap();
    let theme = db::get_user_theme(&db, &user_id).await.unwrap();
    assert_eq!(theme, "pandemonium");
}

#[tokio::test]
async fn test_set_user_theme_invalid_value_rejected() {
    let db = common::test_db().await;
    let user_id = db::create_user(
        &db,
        "theme_user3".into(),
        "theme3@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap();
    // SurrealDB ASSERT must reject 'light' (only demacia/pandemonium allowed)
    let result = db::set_user_theme(&db, &user_id, "light").await;
    assert!(
        result.is_err(),
        "ASSERT in [demacia, pandemonium] should reject 'light'"
    );
}

#[tokio::test]
async fn test_new_user_default_theme_is_demacia() {
    let db = common::test_db().await;
    let user_id = db::create_user(
        &db,
        "theme_user4".into(),
        "theme4@example.com".into(),
        "hash".into(),
    )
    .await
    .unwrap();
    // Newly-created users should fall back to 'demacia' via DEFAULT
    let theme = db::get_user_theme(&db, &user_id).await.unwrap();
    assert_eq!(theme, "demacia");
}
