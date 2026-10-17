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

/// A server function's database handle: the shared handle plus the request
/// path that borrowed it. Live handles are counted per path, so a shutdown
/// that still finds handles can say which requests hold them
/// ([`held_handles`]). Derefs to `Surreal<Db>`.
pub struct DbHandle {
    db: Arc<Surreal<Db>>,
    path: Arc<str>,
    /// The request that borrowed it, for the in-flight steps.
    request: Option<crate::server::inflight::RequestId>,
}

fn holders() -> &'static std::sync::Mutex<std::collections::HashMap<Arc<str>, usize>> {
    static HOLDERS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<Arc<str>, usize>>,
    > = std::sync::OnceLock::new();
    HOLDERS.get_or_init(Default::default)
}

impl DbHandle {
    fn new(db: Arc<Surreal<Db>>, path: Arc<str>) -> Self {
        *holders()
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(Arc::clone(&path))
            .or_insert(0) += 1;
        let request = crate::server::inflight::current();
        if let Some(id) = request {
            crate::server::inflight::note_for(id, "db borrowed");
        }
        Self { db, path, request }
    }

    /// The `Arc` itself, for an API that keeps one (it is not counted).
    pub fn arc(&self) -> Arc<Surreal<Db>> {
        Arc::clone(&self.db)
    }
}

impl Clone for DbHandle {
    fn clone(&self) -> Self {
        Self::new(Arc::clone(&self.db), Arc::clone(&self.path))
    }
}

impl Drop for DbHandle {
    fn drop(&mut self) {
        if let Some(id) = self.request {
            crate::server::inflight::note_for(id, "db released");
        }
        let mut map = holders().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(n) = map.get_mut(&self.path) {
            *n -= 1;
            if *n == 0 {
                map.remove(&self.path);
            }
        }
    }
}

impl std::ops::Deref for DbHandle {
    type Target = Surreal<Db>;
    fn deref(&self) -> &Surreal<Db> {
        &self.db
    }
}

/// The request paths whose server functions still hold a database handle,
/// with how many (logged at shutdown when handles are left).
pub fn held_handles() -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = holders()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .iter()
        .map(|(k, n)| (k.to_string(), *n))
        .collect();
    v.sort();
    v
}

pub const NOT_SIGNED_IN: &str = "Not signed in";
pub const NO_TEAM: &str = "Create or join your team first";
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

/// The DB handle for a server function: a clone of the process-wide handle
/// (`db::shared`), held only for the call. (Not a Leptos context value: a
/// clone in a request's owner could outlive the request and keep the
/// database open at shutdown.)
pub fn db() -> Result<DbHandle, ServerFnError> {
    let db = crate::server::db::shared()
        .ok_or_else(|| internal_error("server fn context", "no DB handle (shutting down)"))?;
    let path: Arc<str> = use_context::<axum::http::request::Parts>()
        .map(|p| p.uri.path().to_string())
        .unwrap_or_else(|| "(no request)".to_string())
        .into();
    Ok(DbHandle::new(db, path))
}

/// The shared Riot API client provided to every server function.
pub fn riot() -> Result<Arc<crate::server::riot::RiotClient>, ServerFnError> {
    use_context::<Arc<crate::server::riot::RiotClient>>()
        .ok_or_else(|| internal_error("server fn context", "no Riot client"))
}

/// The signed-in user, if any. A signed-in request also records the visit
/// (`user.last_seen_at`, at most once an hour per user; see
/// [`last_seen_due`]), except on public pages (share links, legal and
/// sign-in pages, changelog, transparency; `counts_as_visit`).
pub async fn optional_user() -> Result<Option<AppUser>, ServerFnError> {
    let auth: AuthSession = leptos_axum::extract()
        .await
        .map_err(|e| internal_error("extract auth session", e))?;
    crate::server::inflight::note("auth session extracted");
    if let Some(u) = &auth.user {
        let counts = visit_page()
            .as_deref()
            .is_some_and(crate::models::guidance::counts_as_visit);
        if counts && last_seen_due(&u.id, std::time::Instant::now()) {
            let written = match crate::server::db::shared() {
                Some(db) => crate::server::db::touch_last_seen(&db, &u.id)
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                None => Err("no database handle".to_string()),
            };
            crate::server::inflight::note("visit written");
            if let Err(e) = written {
                // Try again on the next request rather than in an hour.
                forget_seen(&u.id);
                tracing::warn!(user = %u.id, error = %e, "could not record the visit");
            }
        }
    }
    Ok(auth.user)
}

/// The page this request belongs to: the request path for a server-rendered
/// page, or the same-origin `Referer` path for a server function call from
/// the browser (`/api/...`). `None` when it cannot be told.
fn visit_page() -> Option<String> {
    let parts = use_context::<axum::http::request::Parts>()?;
    let path = parts.uri.path();
    if !path.starts_with("/api/") {
        return Some(path.to_string());
    }
    let referer = parts
        .headers
        .get(axum::http::header::REFERER)?
        .to_str()
        .ok()?;
    let uri: axum::http::Uri = referer.parse().ok()?;
    Some(uri.path().to_string())
}

fn seen_map() -> &'static std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>> {
    static SEEN: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>,
    > = std::sync::OnceLock::new();
    SEEN.get_or_init(Default::default)
}

/// Forget the in-memory "written" mark (the write failed).
fn forget_seen(user_id: &str) {
    seen_map()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .remove(user_id);
}

/// Whether this process should write the user's visit now: the first
/// signed-in request, then once an hour (the database query checks the hour
/// too, across restarts). Kept in memory so most requests cost nothing.
pub fn last_seen_due(user_id: &str, now: std::time::Instant) -> bool {
    let mut map = seen_map().lock().unwrap_or_else(|p| p.into_inner());
    last_seen_due_in(&mut map, user_id, now)
}

fn last_seen_due_in(
    map: &mut std::collections::HashMap<String, std::time::Instant>,
    user_id: &str,
    now: std::time::Instant,
) -> bool {
    const HOUR: std::time::Duration = std::time::Duration::from_secs(3600);
    match map.get(user_id) {
        Some(at) if now.duration_since(*at) < HOUR => false,
        _ => {
            map.insert(user_id.to_string(), now);
            true
        }
    }
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

/// The user's team, or "Create or join your team first".
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

/// The user must lead `team_id` or be one of its coaches (M148: who may
/// manage the team's invite links). Anyone else on the team gets the same
/// refusal as [`require_team_leader`]; another team's member gets "Not found".
pub async fn require_team_leader_or_coach(
    db: &Surreal<Db>,
    user: &AppUser,
    team_id: &str,
) -> Result<(), ServerFnError> {
    require_team_member(db, user, team_id).await?;
    if db::is_team_leader_or_coach(db, &user.id, team_id)
        .await
        .map_err(server_err)?
    {
        Ok(())
    } else {
        Err(ServerFnError::new(LEADER_ONLY))
    }
}

/// Make the stored admin flag match `ADMIN_EMAILS`: promote listed users,
/// demote users whose address was removed (unless the promote command made
/// them admin). Called at login and registration; failures are logged, never
/// fatal. (Access itself never depends on this running: `AppUser::is_admin`
/// also requires a current listing or a promotion.)
pub async fn sync_admin_flag(db: &Surreal<Db>, user: &AppUser) {
    let listed = user.admin_promoted || crate::server::config::get().is_admin_email(&user.email);
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

    /// The user's own team when they lead it or coach it (invite links only),
    /// or an error. The team always comes from the session.
    pub async fn require_leader_or_coach(&self) -> Result<String, ServerFnError> {
        let team_id = self.require_team().await?;
        require_team_leader_or_coach(&self.db, &self.user, &team_id).await?;
        Ok(team_id)
    }
}

// ---------------------------------------------------------------------------
// Rate limiting
// ---------------------------------------------------------------------------

/// The client's IP: the peer address, or the last `X-Forwarded-For` hop when
/// `TRUST_FORWARDED_FOR` is set (behind a reverse proxy).
pub async fn client_ip() -> String {
    let headers = leptos_axum::extract::<axum::http::HeaderMap>()
        .await
        .unwrap_or_default();
    let peer = leptos_axum::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .await
        .ok()
        .map(|axum::extract::ConnectInfo(addr)| addr);
    client_ip_from(&headers, peer)
}

/// The rate-limit key for a request outside a server function (plain Axum
/// handlers): the last `X-Forwarded-For` hop when `TRUST_FORWARDED_FOR` is
/// set, else the peer address.
pub fn client_ip_from(
    headers: &axum::http::HeaderMap,
    peer: Option<std::net::SocketAddr>,
) -> String {
    if crate::server::config::get().trust_forwarded_for {
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
    match peer {
        Some(addr) => addr.ip().to_string(),
        None => "unknown".to_string(),
    }
}

/// Take a token from `bucket` for this client, or fail with "slow down".
pub async fn rate_limit(bucket: Bucket) -> Result<(), ServerFnError> {
    let ip = client_ip().await;
    Limiter::global()
        .hit(bucket, &ip)
        .map_err(|wait| ServerFnError::new(slow_down_message(wait)))
}

/// Take a token from `bucket` for this signed-in user (keyed by user id,
/// so users behind one address do not share a budget).
pub fn rate_limit_user(bucket: Bucket, user: &AppUser) -> Result<(), ServerFnError> {
    Limiter::global()
        .hit(bucket, &user.id)
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

#[cfg(test)]
mod last_seen_tests {
    use super::last_seen_due_in;
    use std::time::{Duration, Instant};

    #[test]
    fn a_visit_is_written_once_an_hour() {
        let mut map = std::collections::HashMap::new();
        let t0 = Instant::now();
        assert!(last_seen_due_in(&mut map, "user:a", t0));
        assert!(!last_seen_due_in(
            &mut map,
            "user:a",
            t0 + Duration::from_secs(60)
        ));
        assert!(last_seen_due_in(&mut map, "user:b", t0), "per user");
        assert!(!last_seen_due_in(
            &mut map,
            "user:a",
            t0 + Duration::from_secs(3599)
        ));
        assert!(last_seen_due_in(
            &mut map,
            "user:a",
            t0 + Duration::from_secs(3600)
        ));
    }
}
