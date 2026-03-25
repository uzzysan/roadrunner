use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

use roadrunner::config::Config;
use roadrunner::state::AppState;
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

    // Utwórz zunifikowany AppState
    let app_state = AppState::new(pool, ws_state, config);

    // Konfiguracja routingu
    let app = Router::new()
        // Health check
        .route("/", get(root))
        .route("/health", get(health_check))
        // WebSocket
        .route("/ws", get(roadrunner::websocket::ws_handler))
        // Auth
        .route("/auth/register", post(roadrunner::handlers::auth::register))
        .route("/auth/login", post(roadrunner::handlers::auth::login))
        // MFA
        .route(
            "/auth/mfa/setup",
            post(roadrunner::handlers::auth::setup_mfa),
        )
        .route(
            "/auth/mfa/verify-setup",
            post(roadrunner::handlers::auth::verify_mfa_setup),
        )
        .route(
            "/auth/mfa/verify-login",
            post(roadrunner::handlers::auth::verify_mfa_login),
        )
        .route(
            "/auth/mfa/disable",
            post(roadrunner::handlers::auth::disable_mfa),
        )
        // Token Refresh & Logout
        .route(
            "/auth/refresh",
            post(roadrunner::handlers::auth::refresh_token),
        )
        .route("/auth/logout", post(roadrunner::handlers::auth::logout))
        // Tickets
        .route(
            "/tickets",
            post(roadrunner::handlers::tickets::create_ticket),
        )
        .route("/tickets", get(roadrunner::handlers::tickets::list_tickets))
        .route(
            "/tickets/:id",
            get(roadrunner::handlers::tickets::get_ticket),
        )
        .route(
            "/tickets/validate",
            post(roadrunner::handlers::tickets::validate_ticket),
        )
        // Payments
        .route(
            "/payments",
            post(roadrunner::handlers::payments::create_payment),
        )
        .route(
            "/payments",
            get(roadrunner::handlers::payments::list_payments),
        )
        .route(
            "/payments/:id",
            get(roadrunner::handlers::payments::get_payment),
        )
        // Stripe Webhook
        .route(
            "/webhooks/stripe",
            post(roadrunner::handlers::payments::stripe_webhook),
        )
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", app_state.config.port);
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
