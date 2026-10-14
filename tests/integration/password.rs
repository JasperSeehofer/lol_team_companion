//! M137 item 5: password change and admin reset codes. A change ends the
//! account's other sessions; a reset code works once, for 15 minutes, only
//! for the account it was made for, only when an admin made it; deletion
//! removes the account's codes.

use std::collections::HashMap;

use lol_team_companion::server::account::{self, FORMER_MEMBER_ID};
use lol_team_companion::server::auth::session_auth_hash_for;
use lol_team_companion::server::db::{self, DbError};
use lol_team_companion::server::password;
use lol_team_companion::server::session_store::SurrealSessionStore;
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};

struct World {
    admin: String,
    alice: String,
    bob: String,
}

async fn world(db: &Surreal<Db>) -> World {
    let admin = db::create_user(
        db,
        "root".into(),
        "root@example.com".into(),
        "h-admin".into(),
    )
    .await
    .unwrap();
    db::set_user_admin(db, &admin, true).await.unwrap();
    let alice = db::create_user(
        db,
        "alice".into(),
        "alice@example.com".into(),
        "h-alice".into(),
    )
    .await
    .unwrap();
    let bob = db::create_user(db, "bob".into(), "bob@example.com".into(), "h-bob".into())
        .await
        .unwrap();
    World { admin, alice, bob }
}

async fn count(db: &Surreal<Db>, q: &str) -> i64 {
    #[derive(Debug, serde::Deserialize, SurrealValue)]
    struct N {
        n: i64,
    }
    let mut r = db.query(q).await.unwrap();
    let n: Option<N> = r.take(0).unwrap();
    n.map(|n| n.n).unwrap_or(0)
}

async fn hash_of(db: &Surreal<Db>, user: &str) -> String {
    db::password_hash_of(db, user).await.unwrap()
}

#[tokio::test]
async fn changing_the_password_ends_the_other_sessions() {
    use tower_sessions_core::session::{Id, Record};
    use tower_sessions_core::SessionStore;
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let old = hash_of(&db, &w.alice).await;

    let store = SurrealSessionStore::new(db.clone(), "sessions".into());
    let expiry = tower_sessions::cookie::time::OffsetDateTime::now_utc()
        + tower_sessions::cookie::time::Duration::days(1);
    for user in [&w.alice, &w.alice, &w.bob] {
        let mut data = HashMap::new();
        data.insert(
            "axum-login.data".to_string(),
            serde_json::json!({ "user_id": user, "auth_hash": session_auth_hash_for(&old) }),
        );
        let mut record = Record {
            id: Id::default(),
            data,
            expiry_date: expiry,
        };
        store.create(&mut record).await.unwrap();
    }

    password::set_password_hash(&db, &w.alice, "h-alice-new".into())
        .await
        .unwrap();
    assert_eq!(hash_of(&db, &w.alice).await, "h-alice-new");
    // Every old session stops matching on its next request ...
    assert_ne!(
        session_auth_hash_for(&old),
        session_auth_hash_for("h-alice-new")
    );
    // ... and the change deletes the rows (the caller then signs in again).
    assert_eq!(store.delete_for_user(&w.alice).await.unwrap(), 2);
    assert_eq!(
        count(&db, "SELECT count() AS n FROM sessions GROUP ALL").await,
        1,
        "bob's session stays"
    );
    assert_eq!(hash_of(&db, &w.bob).await, "h-bob");
    assert!(matches!(
        password::set_password_hash(&db, "user:nobody", "x".into()).await,
        Err(DbError::NotFound)
    ));
}

#[tokio::test]
async fn a_reset_code_works_once() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let m = password::mint_reset_code(&db, " Alice@Example.com ", Some(&w.admin))
        .await
        .unwrap();
    assert_eq!(m.user_id, w.alice);
    assert_eq!(m.username, "alice");
    // Only the hash is stored.
    let mut r = db
        .query("SELECT VALUE code_hash FROM password_reset")
        .await
        .unwrap();
    let hashes: Vec<String> = r.take(0).unwrap();
    assert_eq!(hashes, vec![password::hash_reset_code(&m.code)]);
    assert!(!hashes[0].contains(&password::normalize_reset_code(&m.code)));

    // Typed in lowercase with spaces still works.
    let typed = m.code.to_lowercase().replace('-', " ");
    let id = password::redeem_reset_code(&db, "alice@example.com", &typed, "h-new".into())
        .await
        .unwrap();
    assert_eq!(id, w.alice);
    assert_eq!(hash_of(&db, &w.alice).await, "h-new");
    assert_eq!(
        count(
            &db,
            "SELECT count() AS n FROM password_reset WHERE used_at != NONE GROUP ALL"
        )
        .await,
        1
    );

    // The second use fails and changes nothing.
    assert!(matches!(
        password::redeem_reset_code(&db, "alice@example.com", &m.code, "h-again".into()).await,
        Err(DbError::NotFound)
    ));
    assert_eq!(hash_of(&db, &w.alice).await, "h-new");

    let rows = password::list_recent_resets(&db, 20).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].username, "alice");
    assert_eq!(rows[0].created_by.as_deref(), Some("root"));
    assert!(rows[0].used_at.is_some());
}

#[tokio::test]
async fn a_wrong_or_expired_code_is_refused() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let m = password::mint_reset_code(&db, "alice@example.com", Some(&w.admin))
        .await
        .unwrap();
    assert!(matches!(
        password::redeem_reset_code(&db, "alice@example.com", "AAAA-BBBB-CCCC-DDDD", "x".into())
            .await,
        Err(DbError::NotFound)
    ));
    db.query("UPDATE password_reset SET expires_at = time::now() - 1m")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert!(matches!(
        password::redeem_reset_code(&db, "alice@example.com", &m.code, "x".into()).await,
        Err(DbError::NotFound)
    ));
    assert_eq!(hash_of(&db, &w.alice).await, "h-alice");
    // The expiry is the database's clock, set at 15 minutes.
    let m = password::mint_reset_code(&db, "alice@example.com", None)
        .await
        .unwrap();
    let exp = chrono::DateTime::parse_from_rfc3339(&m.expires_at).unwrap();
    let left = exp.signed_duration_since(chrono::Utc::now());
    assert!(left <= chrono::Duration::minutes(15) && left > chrono::Duration::minutes(14));
}

#[tokio::test]
async fn a_code_for_one_account_cannot_reset_another() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let m = password::mint_reset_code(&db, "alice@example.com", Some(&w.admin))
        .await
        .unwrap();
    for email in ["bob@example.com", "root@example.com", "nobody@example.com"] {
        assert!(matches!(
            password::redeem_reset_code(&db, email, &m.code, "h-stolen".into()).await,
            Err(DbError::NotFound)
        ));
    }
    assert_eq!(hash_of(&db, &w.bob).await, "h-bob");
    assert_eq!(hash_of(&db, &w.admin).await, "h-admin");
    // The code is still good for its own account.
    password::redeem_reset_code(&db, "alice@example.com", &m.code, "h-new".into())
        .await
        .unwrap();
}

#[tokio::test]
async fn only_an_admin_can_mint_a_code() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    // The server function checks `require_admin` first; the database
    // refuses a creator who is not a stored admin as well.
    assert!(matches!(
        password::mint_reset_code(&db, "alice@example.com", Some(&w.bob)).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        password::mint_reset_code(&db, "alice@example.com", Some(&w.alice)).await,
        Err(DbError::NotFound)
    ));
    assert_eq!(
        count(&db, "SELECT count() AS n FROM password_reset GROUP ALL").await,
        0
    );
    // A promoted admin (the promote command) may.
    db::set_admin_promotion(&db, "bob@example.com", true)
        .await
        .unwrap();
    password::mint_reset_code(&db, "alice@example.com", Some(&w.bob))
        .await
        .unwrap();
    // Unknown accounts and the tombstone get nothing.
    assert!(matches!(
        password::mint_reset_code(&db, "nobody@example.com", Some(&w.admin)).await,
        Err(DbError::NotFound)
    ));
    db.query(
        "UPSERT user:former_member SET username = 'Former member', \
         username_lc = 'former member (account deleted, tombstone)', \
         email = 'former-member.invalid', password_hash = '!', is_admin = false",
    )
    .await
    .unwrap()
    .check()
    .unwrap();
    assert!(matches!(
        password::mint_reset_code(&db, "former-member.invalid", Some(&w.admin)).await,
        Err(DbError::NotFound)
    ));
}

#[tokio::test]
async fn a_new_code_replaces_the_open_one() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let first = password::mint_reset_code(&db, "alice@example.com", Some(&w.admin))
        .await
        .unwrap();
    let second = password::mint_reset_code(&db, "alice@example.com", Some(&w.admin))
        .await
        .unwrap();
    assert_ne!(first.code, second.code);
    assert!(matches!(
        password::redeem_reset_code(&db, "alice@example.com", &first.code, "x".into()).await,
        Err(DbError::NotFound)
    ));
    password::redeem_reset_code(&db, "alice@example.com", &second.code, "h-new".into())
        .await
        .unwrap();
}

#[tokio::test]
async fn concurrent_redeems_of_one_code_succeed_once() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    let m = password::mint_reset_code(&db, "alice@example.com", Some(&w.admin))
        .await
        .unwrap();
    let mut tasks = Vec::new();
    for i in 0..4 {
        let db = db.clone();
        let code = m.code.clone();
        tasks.push(tokio::spawn(async move {
            password::redeem_reset_code(&db, "alice@example.com", &code, format!("h-{i}")).await
        }));
    }
    let mut ok = 0;
    for t in tasks {
        if t.await.unwrap().is_ok() {
            ok += 1;
        }
    }
    assert_eq!(ok, 1);
}

#[tokio::test]
async fn deleting_an_account_removes_its_reset_codes() {
    let db = crate::common::test_db().await;
    let w = world(&db).await;
    password::mint_reset_code(&db, "alice@example.com", Some(&w.admin))
        .await
        .unwrap();
    password::mint_reset_code(&db, "bob@example.com", Some(&w.admin))
        .await
        .unwrap();
    account::delete_account(&db, &w.alice).await.unwrap();
    assert_eq!(
        count(&db, "SELECT count() AS n FROM password_reset GROUP ALL").await,
        1,
        "alice's code is gone, bob's stays"
    );
    // The admin's own deletion keeps bob's code, made by the tombstone.
    account::delete_account(&db, &w.admin).await.unwrap();
    let mut r = db
        .query("SELECT VALUE created_by FROM password_reset")
        .await
        .unwrap();
    let by: Vec<Option<surrealdb::types::RecordId>> = r.take(0).unwrap();
    use surrealdb::types::ToSql;
    assert_eq!(
        by.into_iter()
            .flatten()
            .map(|r| r.to_sql())
            .collect::<Vec<_>>(),
        vec![FORMER_MEMBER_ID.to_string()]
    );
}
