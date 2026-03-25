use async_trait::async_trait;
use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
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
    mode: Option<String>,
    riot_region: Option<String>,
}

impl From<DbUser> for AppUser {
    fn from(u: DbUser) -> Self {
        AppUser {
            id: u.id.to_sql(),
            username: u.username,
            email: u.email,
            password_hash: u.password_hash,
            riot_puuid: u.riot_puuid,
            riot_summoner_name: u.riot_summoner_name,
            mode: u.mode.unwrap_or_else(|| "solo".to_string()),
            riot_region: u.riot_region,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub riot_puuid: Option<String>,
    pub riot_summoner_name: Option<String>,
    pub mode: String,
    pub riot_region: Option<String>,
}

impl AuthUser for AppUser {
    type Id = String;

    fn id(&self) -> String {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
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
    pub db: Arc<Surreal<Db>>,
}

impl AuthBackend {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self { db }
    }
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
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let mut result = self
            .db
            .query("SELECT * FROM user WHERE email = $email LIMIT 1")
            .bind(("email", creds.email))
            .await
            .map_err(AuthError::Db)?;

        let db_user: Option<DbUser> = result.take(0).map_err(AuthError::Db)?;

        let Some(db_user) = db_user else {
            return Ok(None);
        };

        let user: AppUser = db_user.into();

        let hash =
            PasswordHash::new(&user.password_hash).map_err(|e| AuthError::Hash(e.to_string()))?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(creds.password.as_bytes(), &hash)
            .is_ok()
        {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &String) -> Result<Option<Self::User>, Self::Error> {
        let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
        let mut result = self
            .db
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
}

// ---------------------------------------------------------------------------
// Password hashing helper
// ---------------------------------------------------------------------------

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
    fn hashes_are_non_deterministic() {
        let h1 = hash_password("same_password").unwrap();
        let h2 = hash_password("same_password").unwrap();
        assert_ne!(
            h1, h2,
            "Argon2 should produce different hashes due to random salt"
        );
    }
}
