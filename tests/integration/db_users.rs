use crate::common;

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

#[tokio::test]
async fn the_promote_command_promotes_lists_and_revokes() {
    let db = common::test_db().await;
    let id = db::create_user(&db, "ada".into(), "ada@example.com".into(), "h".into())
        .await
        .unwrap();
    db::create_user(&db, "bob".into(), "bob@example.com".into(), "h".into())
        .await
        .unwrap();
    assert!(db::list_admins(&db).await.unwrap().is_empty());

    // The address is matched case-insensitively, like sign-in.
    let (pid, name) = db::set_admin_promotion(&db, " Ada@Example.com ", true)
        .await
        .unwrap();
    assert_eq!((pid.as_str(), name.as_str()), (id.as_str(), "ada"));
    assert_eq!(
        db::list_admins(&db).await.unwrap(),
        vec![("ada@example.com".to_string(), "ada".to_string(), true)]
    );

    db::set_admin_promotion(&db, "ada@example.com", false)
        .await
        .unwrap();
    assert!(db::list_admins(&db).await.unwrap().is_empty());

    match db::set_admin_promotion(&db, "nobody@example.com", true).await {
        Err(db::DbError::NotFound) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
}
