//! Authorization and error helpers for server functions.
//!
//! Every `#[server]` function that touches user or team data starts with
//! `authed().await?` (signed-in user + DB handle) and derives the team on the
//! server with `ctx.require_team()` / `ctx.team()`; it never trusts a team ID
//! sent by the client. DB functions take that team (or the user) as their
//! scope and treat records outside it as "not found".
//!
//! Errors: `server_err` logs the real error and hands the client a short,
//! generic message. Only `DbError::Invalid` (rule violations written for
//! users) and "not found" pass through.

use std::fmt::Display;
use std::sync::Arc;

use leptos::prelude::*;
use surrealdb::{engine::local::Db, Surreal};

use crate::server::auth::{AppUser, AuthSession};
use crate::server::db::{self, DbError};
use crate::server::rate_limit::{slow_down_message, Bucket, Limiter};

pub type DbHandle = Arc<Surreal<Db>>;

pub const NOT_SIGNED_IN: &str = "Not signed in";
pub const NO_TEAM: &str = "Join or create a team first";
pub const NOT_FOUND: &str = "Not found";
pub const LEADER_ONLY: &str = "Only the team leader can do that";
const GENERIC: &str = "Something went wrong on our side. Please try again.";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Log `e` with context and return a generic error for the client.
pub fn internal_error(context: &str, e: impl Display) -> ServerFnError {
    tracing::error!(context, error = %e, "server function failed");
    ServerFnError::new(GENERIC)
}

/// Errors that know how to present themselves to the client.
pub trait IntoServerError {
    fn into_server_error(self) -> ServerFnError;
}

impl IntoServerError for DbError {
    fn into_server_error(self) -> ServerFnError {
        match self {
            DbError::NotFound => ServerFnError::new(NOT_FOUND),
            DbError::Invalid(msg) => ServerFnError::new(msg),
            other => internal_error("database", other),
        }
    }
}

impl IntoServerError for surrealdb::types::Error {
    fn into_server_error(self) -> ServerFnError {
        internal_error("database", self)
    }
}

impl IntoServerError for crate::server::riot::RiotError {
    fn into_server_error(self) -> ServerFnError {
        use crate::server::riot::RiotError;
        match self {
            // Our own messages ("Account X#Y not found"), safe to show.
            RiotError::Api(msg) => ServerFnError::new(msg),
            RiotError::NotConfigured => ServerFnError::new(
                "Riot features are not set up on this server (no API key). Ask an admin.",
            ),
            RiotError::Busy(secs) => {
                tracing::info!(retry_in = secs, "global Riot budget spent");
                ServerFnError::new(format!("Riot is busy, try again in {secs} s."))
            }
            RiotError::Riven(e) => {
                let status = e.status_code().map(|s| s.as_u16());
                tracing::warn!(?status, error = %e, "Riot API request failed");
                ServerFnError::new(match status {
                    Some(404) => "Riot has no data for that request.",
                    Some(429) => "The Riot API is busy. Please try again in a minute.",
                    Some(401) | Some(403) => "The Riot API rejected our key. Please tell an admin.",
                    _ => "The Riot API request failed. Please try again later.",
                })
            }
        }
    }
}

impl IntoServerError for serde_json::Error {
    fn into_server_error(self) -> ServerFnError {
        tracing::debug!(error = %self, "invalid JSON from client");
        ServerFnError::new("Invalid request data")
    }
}

impl IntoServerError for crate::server::data_dragon::DataDragonError {
    fn into_server_error(self) -> ServerFnError {
        tracing::warn!(error = %self, "Data Dragon request failed");
        ServerFnError::new("Champion data is unavailable right now. Please try again later.")
    }
}

impl IntoServerError for axum_login::Error<crate::server::auth::AuthBackend> {
    fn into_server_error(self) -> ServerFnError {
        internal_error("auth session", self)
    }
}

/// `.map_err(server_err)` for any error that implements `IntoServerError`.
pub fn server_err<E: IntoServerError>(e: E) -> ServerFnError {
    e.into_server_error()
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// The DB handle provided to every server function.
pub fn db() -> Result<DbHandle, ServerFnError> {
    use_context::<DbHandle>().ok_or_else(|| internal_error("server fn context", "no DB handle"))
}

/// The shared Riot API client provided to every server function.
pub fn riot() -> Result<Arc<crate::server::riot::RiotClient>, ServerFnError> {
    use_context::<Arc<crate::server::riot::RiotClient>>()
        .ok_or_else(|| internal_error("server fn context", "no Riot client"))
}

/// The signed-in user, if any.
pub async fn optional_user() -> Result<Option<AppUser>, ServerFnError> {
    let auth: AuthSession = leptos_axum::extract()
        .await
        .map_err(|e| internal_error("extract auth session", e))?;
    Ok(auth.user)
}

/// The signed-in user, or "Not signed in".
pub async fn require_user() -> Result<AppUser, ServerFnError> {
    optional_user()
        .await?
        .ok_or_else(|| ServerFnError::new(NOT_SIGNED_IN))
}

/// An admin user. Everyone else gets "Not found", so the admin surface does
/// not reveal that it exists.
pub async fn require_admin() -> Result<AppUser, ServerFnError> {
    match optional_user().await? {
        Some(u) if u.is_admin => Ok(u),
        _ => Err(ServerFnError::new(NOT_FOUND)),
    }
}

/// The user's team, derived on the server.
pub async fn current_team_for(
    db: &Surreal<Db>,
    user: &AppUser,
) -> Result<Option<String>, ServerFnError> {
    db::get_user_team_id(db, &user.id).await.map_err(server_err)
}

/// The user's team, or "Join or create a team first".
pub async fn require_team(db: &Surreal<Db>, user: &AppUser) -> Result<String, ServerFnError> {
    current_team_for(db, user)
        .await?
        .ok_or_else(|| ServerFnError::new(NO_TEAM))
}

/// The user must be a member of `team_id`.
pub async fn require_team_member(
    db: &Surreal<Db>,
    user: &AppUser,
    team_id: &str,
) -> Result<(), ServerFnError> {
    if db::is_team_member(db, &user.id, team_id)
        .await
        .map_err(server_err)?
    {
        Ok(())
    } else {
        Err(ServerFnError::new(NOT_FOUND))
    }
}

/// The user must lead (have created) `team_id`.
pub async fn require_team_leader(
    db: &Surreal<Db>,
    user: &AppUser,
    team_id: &str,
) -> Result<(), ServerFnError> {
    require_team_member(db, user, team_id).await?;
    if db::is_team_leader(db, &user.id, team_id)
        .await
        .map_err(server_err)?
    {
        Ok(())
    } else {
        Err(ServerFnError::new(LEADER_ONLY))
    }
}

/// Make the stored admin flag match `ADMIN_EMAILS`: promote listed users,
/// demote users whose address was removed. Called at login and registration;
/// failures are logged, never fatal. (Access itself never depends on this
/// running: `AppUser::is_admin` also requires a current listing.)
pub async fn sync_admin_flag(db: &Surreal<Db>, user: &AppUser) {
    let listed = crate::server::config::get().is_admin_email(&user.email);
    if listed == user.admin_flag {
        return;
    }
    match db::set_user_admin(db, &user.id, listed).await {
        Ok(()) if listed => tracing::info!(user = %user.id, "promoted to admin via ADMIN_EMAILS"),
        Ok(()) => tracing::info!(user = %user.id, "admin flag removed (not in ADMIN_EMAILS)"),
        Err(e) => tracing::warn!(user = %user.id, error = %e, "admin flag sync failed"),
    }
}

/// A signed-in user plus the DB handle: the start of every protected server fn.
pub struct Ctx {
    pub db: DbHandle,
    pub user: AppUser,
}

pub async fn authed() -> Result<Ctx, ServerFnError> {
    let user = require_user().await?;
    Ok(Ctx { db: db()?, user })
}

impl Ctx {
    pub async fn team(&self) -> Result<Option<String>, ServerFnError> {
        current_team_for(&self.db, &self.user).await
    }

    pub async fn require_team(&self) -> Result<String, ServerFnError> {
        require_team(&self.db, &self.user).await
    }

    /// The team the user leads, or an error.
    pub async fn require_leader(&self) -> Result<String, ServerFnError> {
        let team_id = self.require_team().await?;
        require_team_leader(&self.db, &self.user, &team_id).await?;
        Ok(team_id)
    }
}

// ---------------------------------------------------------------------------
// Rate limiting
// ---------------------------------------------------------------------------

/// The client's IP: the peer address, or the last `X-Forwarded-For` hop when
/// `TRUST_FORWARDED_FOR` is set (behind a reverse proxy).
pub async fn client_ip() -> String {
    if crate::server::config::get().trust_forwarded_for {
        if let Ok(headers) = leptos_axum::extract::<axum::http::HeaderMap>().await {
            if let Some(ip) = headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.rsplit(',').next())
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            {
                return ip;
            }
        }
    }
    match leptos_axum::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>>().await {
        Ok(axum::extract::ConnectInfo(addr)) => addr.ip().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

/// Take a token from `bucket` for this client, or fail with "slow down".
pub async fn rate_limit(bucket: Bucket) -> Result<(), ServerFnError> {
    let ip = client_ip().await;
    Limiter::global()
        .hit(bucket, &ip)
        .map_err(|wait| ServerFnError::new(slow_down_message(wait)))
}

/// Allow `name` for this team at most once per `period`.
pub fn team_cooldown(
    name: &'static str,
    team_id: &str,
    period: std::time::Duration,
) -> Result<(), ServerFnError> {
    Limiter::global()
        .cooldown(name, team_id, period)
        .map_err(|remaining| {
            ServerFnError::new(format!(
                "Your team synced a moment ago. Please wait {} seconds before syncing again.",
                remaining.as_secs().max(1)
            ))
        })
}

/// Allow `name` for this user at most once per `period`.
pub fn user_cooldown(
    name: &'static str,
    user: &AppUser,
    period: std::time::Duration,
) -> Result<(), ServerFnError> {
    Limiter::global()
        .cooldown(name, &user.id, period)
        .map_err(|remaining| {
            ServerFnError::new(format!(
                "You synced a moment ago. Please wait {} seconds before syncing again.",
                remaining.as_secs().max(1)
            ))
        })
}
