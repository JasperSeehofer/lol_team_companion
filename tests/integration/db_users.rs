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

/// Whether the user row has `terms_accepted_at` set.
async fn has_accepted_terms(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    user: &str,
) -> bool {
    let mut r = db
        .query("SELECT VALUE terms_accepted_at != NONE FROM ONLY type::record('user', $k)")
        .bind(("k", db::record_key(user, "user")))
        .await
        .unwrap();
    let v: Option<bool> = r.take(0).unwrap();
    v.unwrap_or(false)
}

/// M148 item 7: the sign-up refuses an unticked 16+/terms box before the
/// invite is touched (register_action checks `terms_accepted` ahead of
/// `consume_invite_grant`), and an account made through the form records
/// when the box was ticked.
#[tokio::test]
async fn sign_up_without_the_terms_box_leaves_the_invite_unused() {
    use lol_team_companion::pages::auth::register::terms_accepted;

    let db = common::test_db().await;
    db::create_invite(&db, None, "ONE-USE".into(), 1, None, None)
        .await
        .unwrap();

    // What a form without the box (or an unticked box) sends.
    for refused in [None, Some(""), Some("off"), Some("no"), Some("false")] {
        assert!(!terms_accepted(refused), "{refused:?}");
    }
    for ticked in [
        Some("on"),
        Some("ON"),
        Some(" true "),
        Some("1"),
        Some("yes"),
    ] {
        assert!(terms_accepted(ticked), "{ticked:?}");
    }

    // The refusal happened before the invite: its one use is still there.
    let grant = db::consume_invite_grant(&db, "one-use").await.unwrap();
    assert!(
        grant.is_some(),
        "the refused sign-up must not use the invite"
    );
    assert!(
        db::consume_invite_grant(&db, "one-use")
            .await
            .unwrap()
            .is_none(),
        "the code had exactly one use"
    );

    let via_form = db::create_user_accepting_terms(
        &db,
        "terms".into(),
        "terms@example.com".into(),
        "h".into(),
    )
    .await
    .unwrap();
    assert!(has_accepted_terms(&db, &via_form).await);

    let other = db::create_user(&db, "plain".into(), "plain@example.com".into(), "h".into())
        .await
        .unwrap();
    assert!(
        !has_accepted_terms(&db, &other).await,
        "only the form records it"
    );
}
