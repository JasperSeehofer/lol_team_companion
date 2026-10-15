#![recursion_limit = "512"]

use axum::{http::StatusCode, response::Redirect, routing::get, Router};
use axum_login::{tower_sessions::SessionManagerLayer, AuthManagerLayerBuilder};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::future::IntoFuture;
use std::sync::Arc;
use std::time::Duration;

use lol_team_companion::app::{shell, App};
use lol_team_companion::server::{
    auth::AuthBackend, config, db, images, rate_limit, riot::RiotClient,
    session_store::SurrealSessionStore,
};
use tower_sessions::ExpiredDeletion;

/// No database handle in here (polish pass): the router's state and the
/// Leptos context closure are cloned into every request, and a request whose
/// render never finished kept the database open at shutdown. Handlers take
/// the handle from `db::shared()` for the length of the call.
#[derive(Clone, axum::extract::FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    riot: Arc<RiotClient>,
    auth_backend: AuthBackend,
}

/// 200 when a trivial DB query succeeds, 503 otherwise.
async fn health_handler() -> (StatusCode, axum::Json<serde_json::Value>) {
    let db_ok = match db::shared() {
        Some(db) => match db.query("RETURN true").await {
            Ok(r) => r.check().is_ok(),
            Err(_) => false,
        },
        None => false,
    };
    if db_ok {
        (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "status": "ok", "db": "ok" })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({ "status": "error", "db": "error" })),
        )
    }
}

/// Resolves on SIGTERM (`docker stop`) or SIGINT (Ctrl+C).
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("cannot listen for Ctrl+C: {e}");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => {
                tracing::error!("cannot listen for SIGTERM: {e}");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("SIGINT received, shutting down"),
        _ = terminate => tracing::info!("SIGTERM received, shutting down"),
    }
}

/// Wait until SurrealKV has released `<data_dir>/LOCK` (the datastore is
/// closed), at most `limit`. SurrealKV holds an exclusive `flock` on that
/// file; taking it here succeeds only once SurrealKV let go (flock locks of
/// two separate opens conflict even inside one process).
async fn wait_for_db_close(data_dir: &std::path::Path, limit: Duration) -> bool {
    let lock_path = data_dir.join("LOCK");
    let deadline = tokio::time::Instant::now() + limit;
    loop {
        match std::fs::File::open(&lock_path) {
            Ok(file) => match file.try_lock() {
                Ok(()) => return true, // unlocked again when `file` drops here
                Err(std::fs::TryLockError::WouldBlock) => {}
                Err(std::fs::TryLockError::Error(e)) => {
                    tracing::warn!("cannot check the database lock: {e}");
                    return false;
                }
            },
            // No LOCK file: nothing holds the directory.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
            Err(e) => {
                tracing::warn!("cannot open {}: {e}", lock_path.display());
                return false;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// `GET /account/export`: everything stored about the signed-in user as a
/// JSON download (`server::account::export_user_data`). 401 when signed out,
/// 429 over the per-user `Bucket::Account` limit. A plain route (not a
/// server function) so the browser saves it as a file; the session cookie is
/// SameSite=Strict, so another site cannot trigger it with the cookie.
async fn export_handler(
    auth: lol_team_companion::server::auth::AuthSession,
) -> axum::response::Response {
    use axum::http::header;
    use axum::response::IntoResponse;
    use lol_team_companion::server::rate_limit::{slow_down_message, Bucket, Limiter};

    let Some(user) = auth.user else {
        return (StatusCode::UNAUTHORIZED, "Sign in to download your data.").into_response();
    };
    if let Err(wait) = Limiter::global().hit(Bucket::Account, &user.id) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, wait.as_secs().max(1).to_string())],
            slow_down_message(wait),
        )
            .into_response();
    }
    let Some(db) = db::shared() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "The server is restarting. Try again in a minute.",
        )
            .into_response();
    };
    match lol_team_companion::server::account::export_user_data(&db, &user.id).await {
        Ok(doc) => {
            let body = serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into());
            let file = format!(
                "attachment; filename=\"companion-data-{}.json\"",
                chrono::Utc::now().format("%Y-%m-%d")
            );
            tracing::info!(user = %user.id, "data export downloaded");
            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        "application/json; charset=utf-8".to_string(),
                    ),
                    (header::CONTENT_DISPOSITION, file),
                    (header::CACHE_CONTROL, "no-store".to_string()),
                ],
                body,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(user = %user.id, "data export failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The export failed. Try again in a moment.",
            )
                .into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenvy::dotenv().ok();

    // `lol_team_companion promote ...`: the admin command (server::promote),
    // so the container image can run it without a second binary. It opens
    // the database itself; the server must be stopped.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("promote") {
        std::process::exit(lol_team_companion::server::promote::run(&args[1..]).await);
    }
    // `lol_team_companion reset-password <email>`: a one-time password reset
    // code (server::reset_password); the server must be stopped as well.
    if args.first().map(String::as_str) == Some("reset-password") {
        std::process::exit(lol_team_companion::server::reset_password::run(&args[1..]).await);
    }

    // Logging. An invalid RUST_LOG is reported, not silently replaced.
    let filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(f) => f,
        Err(e) => {
            if std::env::var_os("RUST_LOG").is_some() {
                eprintln!("RUST_LOG is invalid ({e}); using the default filter");
            }
            tracing_subscriber::EnvFilter::new(config::DEFAULT_RUST_LOG)
        }
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // All configuration, validated once: every error is listed, then the
    // server stops (config::AppConfig, the table in config::VARIABLES).
    let app_config = match config::AppConfig::from_env() {
        Ok((c, warnings)) => {
            for w in &warnings {
                tracing::warn!("Configuration: {w}");
            }
            c
        }
        Err(errors) => {
            for e in &errors {
                tracing::error!("Configuration error: {e}");
            }
            tracing::error!(
                "{} configuration error(s); the server does not start",
                errors.len()
            );
            std::process::exit(1);
        }
    };
    let security = app_config.security.clone();
    tracing::info!(
        dev = security.is_dev,
        closed_beta = security.closed_beta,
        cookie_secure = security.cookie_secure,
        admin_emails = security.admin_emails.len(),
        trust_forwarded_for = security.trust_forwarded_for,
        rate_limit_multiplier = security.rate_limit_multiplier,
        riot_key = app_config.riot.api_key.is_some(),
        riot_budget_per_sec = app_config.riot.budget_per_sec,
        riot_budget_per_min = app_config.riot.budget_per_min,
        match_cache_retention_days = app_config.match_cache_retention_days,
        data_dir = %app_config.data_dir.display(),
        "security configuration"
    );
    if !security.closed_beta {
        tracing::warn!("Invite gate is OFF (CLOSED_BETA=false in dev): anyone can register");
    }
    let bootstrap_invite = security.bootstrap_invite.clone();
    let cookie_secure = security.cookie_secure;
    config::init(security);

    // SurrealDB
    let data_dir = app_config.data_dir.to_string_lossy().into_owned();
    // Applies schema.surql and any pending migrations; a failed migration
    // stops the server here rather than serving a half-migrated database.
    let surreal_db = match db::init_db(&data_dir).await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Database initialisation failed: {e}");
            std::process::exit(1);
        }
    };

    // Server functions borrow the handle from one process-wide slot.
    db::install_shared(Arc::clone(&surreal_db));

    // Data Dragon: load the stored copy, refresh from the network, then keep
    // refreshing every 6 hours. Champion names are normalised against it.
    lol_team_companion::server::data_dragon::init(&surreal_db).await;
    // Draft Graph trait table: shipped defaults for missing champions, then
    // into memory. A failure leaves the graph empty, never the server down.
    if let Err(e) = lol_team_companion::server::graph::traits::sync(&surreal_db).await {
        tracing::error!("champion traits: sync failed: {e}");
    }
    if let Err(e) = db::migrate_champion_names(&surreal_db).await {
        tracing::warn!("Champion name normalisation failed: {e}");
    }
    let ddragon_task =
        lol_team_companion::server::data_dragon::spawn_refresh_task(Arc::clone(&surreal_db));

    // One Riot client for the whole process (shared rate limiting + budget).
    // (A missing key is among the configuration warnings above.)
    let riot = Arc::new(RiotClient::from_config(&app_config.riot));

    // Log requests that do not complete (server::inflight).
    let inflight_task = lol_team_companion::server::inflight::spawn_watchdog();

    // Drop idle rate-limiter entries off the request path.
    let limiter_task = rate_limit::spawn_purge_task(Duration::from_secs(60));

    // Export the feedback (bug-reports.md and feedback.jsonl beside it)
    // before serving, then after every change (at most once a second).
    // Write failures are logged and swallowed.
    let inbox_path = app_config.bug_inbox_path.clone();
    match lol_team_companion::server::bug_report_export::export_feedback(&surreal_db, &inbox_path)
        .await
    {
        Ok(()) => tracing::info!("Feedback exported to {}", inbox_path.display()),
        Err(e) => tracing::warn!("Bug-report inbox export failed: {e}"),
    }
    let feedback_task = lol_team_companion::server::bug_report_export::spawn_exporter(
        db::WeakDb::new(&surreal_db),
        inbox_path,
    );

    if let Some((code, max_uses)) = bootstrap_invite {
        match db::ensure_invite(&surreal_db, &code, max_uses).await {
            Ok(()) => tracing::info!("Bootstrap invite code is available"),
            Err(e) => tracing::warn!("Could not create the bootstrap invite code: {e}"),
        }
    }

    // Leptos config
    let conf = match get_configuration(None) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Configuration error (LEPTOS_* variables): {e}");
            std::process::exit(1);
        }
    };
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Sessions
    let session_store = SurrealSessionStore::new(Arc::clone(&surreal_db), "sessions".to_string());

    // Purge expired sessions, and team notes deleted longer ago than their
    // Undo window, now and every hour.
    let session_purge_task = {
        let store = session_store.clone();
        let db = Arc::clone(&surreal_db);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Err(e) = store.delete_expired().await {
                    tracing::warn!("Session purge failed: {e}");
                }
                if let Err(e) = db::purge_deleted_team_notes(&db).await {
                    tracing::warn!("Deleted team note purge failed: {e}");
                }
                if let Err(e) = lol_team_companion::server::password::purge_old_resets(&db).await {
                    tracing::warn!("Password reset purge failed: {e}");
                }
            }
        })
    };

    // Retention of the shared Riot match caches (MATCH_CACHE_RETENTION_DAYS,
    // twelve months by default): a minute after start, then every 24 hours.
    // The backup restarts the app nightly, so in practice it runs each night.
    let match_cache_task = {
        let db = Arc::clone(&surreal_db);
        let days = app_config.match_cache_retention_days;
        tokio::spawn(async move {
            let day = Duration::from_secs(24 * 3600);
            let first = tokio::time::Instant::now() + Duration::from_secs(60);
            let mut interval = tokio::time::interval_at(first, day);
            loop {
                interval.tick().await;
                let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
                match db::purge_match_cache(&db, cutoff, db::MATCH_CACHE_BATCH).await {
                    Ok(p) => tracing::info!(
                        retention_days = days,
                        details = p.details,
                        timeline_events = p.timeline_events,
                        matches = p.matches,
                        "match cache retention: deleted games cached before {}",
                        cutoff.format("%Y-%m-%d")
                    ),
                    Err(e) => tracing::warn!("match cache retention failed: {e}"),
                }
                // Ranked snapshots (the LP history, one per sync) follow the
                // same retention (M141).
                match db::purge_ranked_snapshots(&db, cutoff, db::MATCH_CACHE_BATCH).await {
                    Ok(n) => tracing::info!(
                        retention_days = days,
                        ranked_snapshots = n,
                        "ranked snapshot retention: deleted snapshots taken before {}",
                        cutoff.format("%Y-%m-%d")
                    ),
                    Err(e) => tracing::warn!("ranked snapshot retention failed: {e}"),
                }
            }
        })
    };

    // SameSite=Strict and HttpOnly are the tower-sessions defaults; Secure
    // comes from config (on unless dev).
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(cookie_secure)
        .with_expiry(tower_sessions::Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::days(30),
        ));

    // Auth
    let auth_backend = AuthBackend::new(Arc::clone(&surreal_db));
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend.clone(), session_layer).build();

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        riot: Arc::clone(&riot),
        auth_backend,
    };

    let app = Router::new()
        .route("/healthz", get(health_handler))
        .route("/account/export", get(export_handler))
        // Champion and item images, cached server-side (never a browser CDN call).
        .route("/img/champion/{key}", get(images::champion_portrait))
        .route("/img/champion/{key}/icon", get(images::champion_icon))
        .route("/img/item/{id}", get(images::item_icon))
        // Parked pages and the old solo dashboard redirect here, in axum: a
        // Leptos `<Redirect>` rendered on the server left its 302 status
        // behind for later static-file responses on the same worker thread.
        .route("/solo", get(|| async { Redirect::temporary("/me") }))
        .route(
            "/analytics",
            get(|| async { Redirect::temporary("/stats") }),
        )
        .route(
            "/team-builder",
            get(|| async { Redirect::temporary("/draft") }),
        )
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                // Only the Riot client: the closure is cloned into every
                // request. The database handle is not a context value (it
                // would live as long as the request's owner); server
                // functions take it from `db::shared()`.
                let riot = Arc::clone(&riot);
                move || {
                    provide_context(Arc::clone(&riot));
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(auth_layer)
        // Outermost: every request is registered until its body is sent, and
        // the watchdog logs any that runs longer than 15 s (M11).
        .layer(axum::middleware::from_fn(
            lol_team_companion::server::inflight::track,
        ))
        .with_state(app_state);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("cannot listen on {addr}: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("Listening on http://{}", addr);

    // Graceful shutdown: on SIGTERM/SIGINT stop accepting, let requests in
    // flight finish for at most `drain`, drop every database handle, wait at
    // most `db_close` for SurrealKV to close, then exit without tearing the
    // runtime down (that waits without a limit for blocking work).
    let drain_limit = app_config.shutdown.drain;
    let close_limit = app_config.shutdown.db_close;
    let (signalled_tx, mut signalled_rx) = tokio::sync::watch::channel(false);
    // Connect info gives server functions the peer IP for rate limiting.
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown_signal().await;
        let _ = signalled_tx.send(true);
    });
    let drain_deadline = async move {
        let _ = signalled_rx.wait_for(|s| *s).await;
        tokio::time::sleep(drain_limit).await;
    };

    // `server` owns the router (and with it AppState, the auth backend and
    // the session store, each holding a database handle); leaving this
    // select drops it, finished or not.
    tokio::select! {
        result = server.into_future() => {
            if let Err(e) = result {
                tracing::error!("server error: {e}");
            }
            tracing::info!("all connections closed");
        }
        _ = drain_deadline => {
            tracing::warn!(
                "requests still running after {}s; closing anyway",
                drain_limit.as_secs()
            );
        }
    }

    // Drop the remaining handles so the embedded engine sees its channel
    // close and shuts the datastore down (flush, close, unlock).
    for task in [
        ddragon_task,
        session_purge_task,
        match_cache_task,
        limiter_task,
        inflight_task,
        feedback_task,
    ] {
        task.abort();
        let _ = task.await;
    }
    drop(riot);
    drop(db::release_shared());
    // Every other handle should be gone now; one still held somewhere (a
    // request's reactive owner, a task) keeps SurrealKV open, so say so.
    let held = Arc::strong_count(&surreal_db) - 1;
    if held > 0 {
        tracing::warn!("{held} database handle(s) still held at shutdown");
        for (path, n) in lol_team_companion::server::authz::held_handles() {
            tracing::warn!("  {n} held by a server function of {path}");
        }
    }
    drop(surreal_db);

    let closed = wait_for_db_close(std::path::Path::new(&data_dir), close_limit).await;
    if closed {
        tracing::info!("database closed; exiting");
    } else {
        // Committed data is on disk already (SurrealKV syncs every commit);
        // the next start replays the log.
        tracing::warn!(
            "database did not close within {}s; exiting anyway",
            close_limit.as_secs()
        );
    }
    std::process::exit(if closed { 0 } else { 1 });
}
