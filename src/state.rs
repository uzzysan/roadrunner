use sqlx::PgPool;
use std::sync::Arc;
use crate::config::Config;
use crate::websocket::state::WsState;

/// Zunifikowany stan aplikacji
/// 
/// Łączy wszystkie zależności w jeden struct dla łatwiejszego zarządzania
#[derive(Clone)]
pub struct AppState {
    /// Połączenie do bazy danych PostgreSQL
    pub db: PgPool,
    /// Stan WebSocket do broadcastowania wiadomości
    pub ws: Arc<WsState>,
    /// Konfiguracja aplikacji
    pub config: Arc<Config>,
}

impl AppState {
    /// Tworzy nowy AppState z podanych komponentów
    pub fn new(db: PgPool, ws: Arc<WsState>, config: Arc<Config>) -> Self {
        Self { db, ws, config }
    }
}
