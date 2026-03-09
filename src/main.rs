use axum::Router;
use axum_login::{AuthManagerLayerBuilder, tower_sessions::SessionManagerLayer};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::sync::Arc;

use lol_team_companion::app::{shell, App};
use lol_team_companion::server::{auth::AuthBackend, db, session_store::SurrealSessionStore};

#[derive(Clone, axum::extract::FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    db: Arc<surrealdb::Surreal<surrealdb::engine::local::Db>>,
    auth_backend: AuthBackend,
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
    let session_store = SurrealSessionStore::new(
        Arc::clone(&surreal_db),
        "sessions".to_string(),
    );
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
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || {
                    let db = Arc::clone(&app_state.db);
                    provide_context(db);
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(auth_layer)
        .with_state(app_state);

    tracing::info!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
