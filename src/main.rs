use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

use roadrunner::config::Config;
use roadrunner::websocket::state::WsState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting RoadRunner server...");

    let config = Arc::new(Config::from_env());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    info!("Connected to database");
    
    let ws_state = Arc::new(WsState::new());
    info!("WebSocket state initialized");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/ws", get(roadrunner::websocket::ws_handler))
        .with_state(ws_state.clone())
        .route("/auth/register", post(roadrunner::handlers::auth::register))
        .route("/auth/login", post(roadrunner::handlers::auth::login))
        .with_state(pool);

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "RoadRunner API - System Transportu Zbiorowego i Szkolnego"
}

async fn health_check() -> &'static str {
    "OK"
}
