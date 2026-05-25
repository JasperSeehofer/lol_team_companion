#![recursion_limit = "512"]

use axum::{extract::State, routing::get, Router};
use axum_login::{tower_sessions::SessionManagerLayer, AuthManagerLayerBuilder};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::sync::Arc;

use lol_team_companion::app::{shell, App};
use lol_team_companion::server::{
    auth::AuthBackend,
    db,
    session_store::SurrealSessionStore,
    theme_layer::{theme_injection_middleware, REQUEST_THEME},
};

#[derive(Clone, axum::extract::FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    db: Arc<surrealdb::Surreal<surrealdb::engine::local::Db>>,
    auth_backend: AuthBackend,
}

async fn health_handler(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let db_ok = state.db.query("RETURN true").await.is_ok();
    axum::Json(serde_json::json!({
        "status": "ok",
        "db": if db_ok { "ok" } else { "error" }
    }))
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

    // SurrealDB
    let data_dir = std::env::var("SURREAL_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let surreal_db = db::init_db(&data_dir)
        .await
        .expect("Failed to initialize SurrealDB");

    // Leptos config
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Sessions
    let session_store = SurrealSessionStore::new(Arc::clone(&surreal_db), "sessions".to_string());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::days(30),
        ));

    // Auth
    let auth_backend = AuthBackend::new(Arc::clone(&surreal_db));
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend.clone(), session_layer).build();

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        db: Arc::clone(&surreal_db),
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
                    // Phase 18.1 plan 01: per-request InitialTheme,
                    // read synchronously from the REQUEST_THEME tokio
                    // task-local that `theme_injection_middleware` set
                    // up earlier in the request lifecycle (after
                    // auth_layer, before this closure runs). Falls
                    // back to InitialTheme::default() ("demacia") when
                    // the task-local is unset (e.g. requests that
                    // bypass the leptos routes, like /healthz).
                    let theme = REQUEST_THEME
                        .try_with(|t| t.clone())
                        .unwrap_or_default();
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
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
