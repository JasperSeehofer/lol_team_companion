#![recursion_limit = "512"]

use axum::{extract::State, http::StatusCode, routing::get, Router};
use axum_login::{tower_sessions::SessionManagerLayer, AuthManagerLayerBuilder};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::sync::Arc;

use lol_team_companion::app::{shell, App};
use lol_team_companion::server::{
    auth::AuthBackend,
    config, db, rate_limit,
    riot::RiotClient,
    session_store::SurrealSessionStore,
    theme_layer::{theme_injection_middleware, REQUEST_THEME},
};
use tower_sessions::ExpiredDeletion;

#[derive(Clone, axum::extract::FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    db: Arc<surrealdb::Surreal<surrealdb::engine::local::Db>>,
    riot: Arc<RiotClient>,
    auth_backend: AuthBackend,
}

/// 200 when a trivial DB query succeeds, 503 otherwise.
async fn health_handler(
    State(state): State<AppState>,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    let db_ok = match state.db.query("RETURN true").await {
        Ok(r) => r.check().is_ok(),
        Err(_) => false,
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

#[tokio::main]
async fn main() {
    // Load .env file (if present)
    dotenvy::dotenv().ok();

    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,lol_team_companion=debug".parse().unwrap()),
        )
        .init();

    // Security configuration (fails fast outside dev without SESSION_SECRET)
    let security = match config::SecurityConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Configuration error: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!(
        dev = security.is_dev,
        closed_beta = security.closed_beta,
        cookie_secure = security.cookie_secure,
        admin_emails = security.admin_emails.len(),
        "security configuration"
    );
    if !security.closed_beta {
        tracing::warn!("Invite gate is OFF (CLOSED_BETA=false in dev): anyone can register");
    }
    let bootstrap_invite = security.bootstrap_invite.clone();
    let cookie_secure = security.cookie_secure;
    config::init(security);

    // SurrealDB
    let data_dir = std::env::var("SURREAL_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    // Applies schema.surql and any pending migrations; a failed migration
    // stops the server here rather than serving a half-migrated database.
    let surreal_db = match db::init_db(&data_dir).await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Database initialisation failed: {e}");
            std::process::exit(1);
        }
    };

    // Data Dragon: load the stored copy, refresh from the network, then keep
    // refreshing every 6 hours. Champion names are normalised against it.
    lol_team_companion::server::data_dragon::init(&surreal_db).await;
    if let Err(e) = db::migrate_champion_names(&surreal_db).await {
        tracing::warn!("Champion name normalisation failed: {e}");
    }
    lol_team_companion::server::data_dragon::spawn_refresh_task(Arc::clone(&surreal_db));

    // One Riot client for the whole process (shared rate limiting + budget).
    let riot = Arc::new(RiotClient::from_env());
    if !riot.is_configured() {
        tracing::warn!("RIOT_API_KEY is not set: Riot features will report that they are off");
    }

    // Drop idle rate-limiter entries off the request path.
    rate_limit::spawn_purge_task(std::time::Duration::from_secs(60));

    // Phase 19 D-04 — auto-export open bug reports to the triage inbox.
    // Runs once, synchronously, after DB init and before axum::serve.
    // Write failures are logged and swallowed (D-04.5).
    let inbox_path = std::env::var("BUG_REPORT_INBOX_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(
                lol_team_companion::server::bug_report_export::DEFAULT_INBOX_PATH,
            )
        });
    if let Err(e) =
        lol_team_companion::server::bug_report_export::export_open_reports(&surreal_db, &inbox_path)
            .await
    {
        tracing::warn!("Bug-report inbox export failed: {e}");
    }

    if let Some((code, max_uses)) = bootstrap_invite {
        match db::ensure_invite(&surreal_db, &code, max_uses).await {
            Ok(()) => tracing::info!("Bootstrap invite code is available"),
            Err(e) => tracing::warn!("Could not create the bootstrap invite code: {e}"),
        }
    }

    // Leptos config
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Sessions
    let session_store = SurrealSessionStore::new(Arc::clone(&surreal_db), "sessions".to_string());

    // Purge expired sessions now and every hour.
    {
        let store = session_store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Err(e) = store.delete_expired().await {
                    tracing::warn!("Session purge failed: {e}");
                }
            }
        });
    }

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
        db: Arc::clone(&surreal_db),
        riot: Arc::clone(&riot),
        auth_backend,
    };

    let app = Router::new()
        .route("/healthz", get(health_handler))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || {
                    let db = Arc::clone(&app_state.db);
                    provide_context(db);
                    provide_context(Arc::clone(&app_state.riot));
                    // Phase 18.1 plan 01: per-request InitialTheme,
                    // read synchronously from the REQUEST_THEME tokio
                    // task-local that `theme_injection_middleware` set
                    // up earlier in the request lifecycle (after
                    // auth_layer, before this closure runs). Falls
                    // back to InitialTheme::default() ("demacia") when
                    // the task-local is unset (e.g. requests that
                    // bypass the leptos routes, like /healthz).
                    let theme = REQUEST_THEME.try_with(|t| t.clone()).unwrap_or_default();
                    provide_context(theme);
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        // Phase 18.1: per-request theme injection middleware.
        //
        // Axum tower layering applies inside-out: the LAST `.layer()`
        // call wraps the OUTERMOST middleware and runs FIRST on each
        // request. We want auth_layer to run FIRST so AuthSession is
        // in extensions when theme_injection_middleware reads it
        // (D-03). Therefore theme_layer goes BEFORE auth_layer in the
        // builder so it sits INSIDE auth_layer's wrap and runs SECOND.
        .layer(axum::middleware::from_fn(theme_injection_middleware))
        .layer(auth_layer)
        .with_state(app_state);

    tracing::info!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // Connect info gives server functions the peer IP for rate limiting.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
