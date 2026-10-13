use async_trait::async_trait;
use axum_login::AuthUser;
use serde::Deserialize;
use std::sync::Arc;
use surrealdb::{
    engine::local::Db,
    types::{RecordId, SurrealValue, ToSql},
    Surreal,
};

// ---------------------------------------------------------------------------
// AuthUser implementation
// ---------------------------------------------------------------------------

/// Internal struct that matches SurrealDB's return format (id as RecordId)
#[derive(Clone, Debug, Deserialize, SurrealValue)]
struct DbUser {
    id: RecordId,
    username: String,
    email: String,
    password_hash: String,
    riot_puuid: Option<String>,
    riot_summoner_name: Option<String>,
    riot_region: Option<String>,
    #[serde(default)]
    is_admin: Option<bool>,
    #[serde(default)]
    admin_promoted_at: Option<surrealdb::types::Datetime>,
}

impl From<DbUser> for AppUser {
    fn from(u: DbUser) -> Self {
        let admin_flag = u.is_admin.unwrap_or(false);
        let admin_promoted = u.admin_promoted_at.is_some();
        AppUser {
            auth_hash: session_auth_hash_for(&u.password_hash),
            is_admin: admin_promoted
                || crate::server::config::get().grants_admin(admin_flag, &u.email),
            admin_flag,
            admin_promoted,
            id: u.id.to_sql(),
            username: u.username,
            email: u.email,
            riot_puuid: u.riot_puuid,
            riot_summoner_name: u.riot_summoner_name,
            riot_region: u.riot_region,
        }
    }
}

/// The signed-in user, loaded from the DB on every request. It carries no
/// password hash; `auth_hash` is an HMAC of it (see `session_auth_hash_for`).
#[derive(Clone)]
pub struct AppUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub riot_puuid: Option<String>,
    pub riot_summoner_name: Option<String>,
    pub riot_region: Option<String>,
    /// Effective admin status: promoted with the promote command, or the
    /// stored flag AND a current `ADMIN_EMAILS` listing, so removing an
    /// address revokes access on the next request.
    pub is_admin: bool,
    /// The raw `user.is_admin` flag as stored (see `authz::sync_admin_flag`).
    pub(crate) admin_flag: bool,
    /// Promoted with the promote command (`user.admin_promoted_at` is set):
    /// admin regardless of `ADMIN_EMAILS` until revoked with the command.
    pub(crate) admin_promoted: bool,
    auth_hash: Vec<u8>,
}

impl std::fmt::Debug for AppUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppUser")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("is_admin", &self.is_admin)
            .finish_non_exhaustive()
    }
}

impl AuthUser for AppUser {
    type Id = String;

    fn id(&self) -> String {
        self.id.clone()
    }

    /// axum-login stores this in the session and compares it on every
    /// request, so a password change ends existing sessions. It is an HMAC of
    /// the password hash keyed with `SESSION_SECRET`, never the hash itself,
    /// so session rows do not contain password material.
    fn session_auth_hash(&self) -> &[u8] {
        &self.auth_hash
    }
}

/// HMAC-SHA256(SESSION_SECRET, password_hash).
pub fn session_auth_hash_for(password_hash: &str) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    let secret = &crate::server::config::get().session_secret;
    let mut mac = <Hmac<sha2::Sha256> as Mac>::new_from_slice(secret)
        .expect("HMAC accepts keys of any length");
    mac.update(b"companion-session-v1:");
    mac.update(password_hash.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

// ---------------------------------------------------------------------------
// Credentials
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

// ---------------------------------------------------------------------------
// AuthnBackend
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AuthBackend {
    /// Not an `Arc`: axum-login clones the backend into every request's
    /// `AuthSession`, and a request that outlives the server must not keep
    /// the database open (polish pass, shutdown).
    db: crate::server::db::WeakDb,
}

impl AuthBackend {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self {
            db: crate::server::db::WeakDb::new(&db),
        }
    }

    fn db(&self) -> Result<Arc<Surreal<Db>>, AuthError> {
        self.db.get().ok_or(AuthError::Closed)
    }
}

/// A valid Argon2 hash of a random password. Unknown emails are verified
/// against it so a login attempt costs the same time whether or not the
/// account exists.
fn dummy_hash() -> &'static str {
    static DUMMY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    DUMMY.get_or_init(|| hash_password("companion-dummy-password").unwrap_or_default())
}

/// Verify a password on the blocking pool (Argon2 is CPU- and memory-heavy and
/// must not run on a Tokio worker).
pub async fn verify_password_async(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        match PasswordHash::new(&hash) {
            Ok(parsed) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    })
    .await
    .unwrap_or(false)
}

#[async_trait]
impl axum_login::AuthnBackend for AuthBackend {
    type User = AppUser;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let email = crate::server::db::normalize_email(&creds.email);

        // Emails are stored lowercase (migration 1 lowercased older rows), so
        // the unique email index answers this exact match.
        let mut result = self
            .db()?
            .query("SELECT * FROM user WHERE email = $email LIMIT 1")
            .bind(("email", email))
            .await
            .map_err(AuthError::Db)?;

        let db_user: Option<DbUser> = result.take(0).map_err(AuthError::Db)?;

        let Some(db_user) = db_user else {
            let _ = verify_password_async(creds.password, dummy_hash().to_string()).await;
            return Ok(None);
        };

        if verify_password_async(creds.password, db_user.password_hash.clone()).await {
            Ok(Some(db_user.into()))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &String) -> Result<Option<Self::User>, Self::Error> {
        let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
        let mut result = self
            .db()?
            .query("SELECT * FROM type::record('user', $user_key) LIMIT 1")
            .bind(("user_key", user_key))
            .await
            .map_err(AuthError::Db)?;

        let db_user: Option<DbUser> = result.take(0).map_err(AuthError::Db)?;
        Ok(db_user.map(|u| u.into()))
    }
}

// ---------------------------------------------------------------------------
// AuthSession type alias
// ---------------------------------------------------------------------------

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Db(#[from] surrealdb::types::Error),
    #[error("Password hash error: {0}")]
    Hash(String),
    #[error("The database is closed (shutting down)")]
    Closed,
}

// ---------------------------------------------------------------------------
// Password hashing helper
// ---------------------------------------------------------------------------

/// Hash a password on the blocking pool.
pub async fn hash_password_async(password: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| e.to_string())?
}

/// Synchronous Argon2 hash. Call `hash_password_async` from async code.
pub fn hash_password(password: &str) -> Result<String, String> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_password() {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        let hash = hash_password("secret123").unwrap();
        let parsed = PasswordHash::new(&hash).unwrap();
        assert!(Argon2::default()
            .verify_password(b"secret123", &parsed)
            .is_ok());
    }

    #[test]
    fn wrong_password_fails_verification() {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        let hash = hash_password("correct_password").unwrap();
        let parsed = PasswordHash::new(&hash).unwrap();
        assert!(Argon2::default()
            .verify_password(b"wrong_password", &parsed)
            .is_err());
    }

    #[test]
    fn session_auth_hash_is_an_hmac_not_the_hash() {
        let hash = hash_password("secret123").unwrap();
        let a = session_auth_hash_for(&hash);
        assert_eq!(a.len(), 32);
        assert_ne!(a, hash.as_bytes());
        assert_eq!(a, session_auth_hash_for(&hash), "deterministic");
        let other = hash_password("secret123").unwrap();
        assert_ne!(a, session_auth_hash_for(&other), "changes with the hash");
    }

    #[tokio::test]
    async fn async_verify_matches_sync() {
        let hash = hash_password_async("correct horse".into()).await.unwrap();
        assert!(verify_password_async("correct horse".into(), hash.clone()).await);
        assert!(!verify_password_async("wrong horse".into(), hash).await);
        assert!(!verify_password_async("x".into(), "not-a-hash".into()).await);
    }

    #[test]
    fn hashes_are_non_deterministic() {
        let h1 = hash_password("same_password").unwrap();
        let h2 = hash_password("same_password").unwrap();
        assert_ne!(
            h1, h2,
            "Argon2 should produce different hashes due to random salt"
        );
    }
}
