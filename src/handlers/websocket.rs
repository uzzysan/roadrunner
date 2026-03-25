//! WebSocket server dla real-time GPS tracking
//!
//! Endpointy WebSocket:
//! - /ws/vehicles - stream lokalizacji wszystkich aktywnych pojazdów
//! - /ws/route/:id - stream dla konkretnej trasy
//! - /ws/vehicle/:id - stream dla konkretnego pojazdu

use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, Path, State},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::state::AppState;

/// Kanał broadcast dla aktualizacji lokalizacji pojazdów
pub type VehicleUpdateSender = broadcast::Sender<VehicleLocationUpdate>;
pub type VehicleUpdateReceiver = broadcast::Receiver<VehicleLocationUpdate>;

/// Aktualizacja lokalizacji pojazdu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLocationUpdate {
    pub vehicle_id: Uuid,
    pub registration_number: String,
    pub route_id: Option<Uuid>,
    pub route_number: Option<String>,
    pub route_color: Option<String>,
    pub driver_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
    pub next_stop_id: Option<Uuid>,
    pub next_stop_name: Option<String>,
    pub eta_seconds: Option<i32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Wiadomość od klienta WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "subscribe_route")]
    SubscribeRoute { route_id: Uuid },
    #[serde(rename = "unsubscribe_route")]
    UnsubscribeRoute,
    #[serde(rename = "ping")]
    Ping,
}

/// Wiadomość do klienta WebSocket
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "vehicle_location")]
    VehicleLocation(VehicleLocationUpdate),
    #[serde(rename = "incident")]
    IncidentNotification(IncidentWebSocketNotification),
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error { message: String },
}

/// Powiadomienie o incydencie przez WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentWebSocketNotification {
    pub incident_id: Uuid,
    pub route_id: Uuid,
    pub route_number: String,
    pub severity: String,
    pub message_pl: String,
    pub message_en: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Handler dla WebSocket - /ws/vehicles
pub async fn ws_vehicles_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_vehicles_socket(socket, state))
}

/// Handler dla WebSocket - /ws/route/:id
pub async fn ws_route_handler(
    ws: WebSocketUpgrade,
    Path(route_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_route_socket(socket, route_id, state))
}

/// Handler dla WebSocket - /ws/vehicle/:id
pub async fn ws_vehicle_handler(
    ws: WebSocketUpgrade,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_vehicle_socket(socket, vehicle_id, state))
}

/// Obsługa połączenia WebSocket dla wszystkich pojazdów
async fn handle_vehicles_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subskrybuj aktualizacje lokalizacji
    let mut vehicle_rx = state.vehicle_updates.subscribe();

    // Task do wysyłania aktualizacji do klienta
    let mut send_task = tokio::spawn(async move {
        loop {
            match vehicle_rx.recv().await {
                Ok(update) => {
                    let msg = ServerMessage::VehicleLocation(update);
                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::error!("Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("WebSocket client lagged behind");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Task do odbierania wiadomości od klienta
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Ping) => {
                            // Pong jest wysyłany automatycznie
                        }
                        Ok(_) => {
                            // Inne komendy - obsługa w zależności od potrzeb
                        }
                        Err(e) => {
                            tracing::warn!("Invalid client message: {}", e);
                        }
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    // Czekaj na zakończenie któregokolwiek taska
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::debug!("WebSocket connection closed");
}

/// Obsługa połączenia WebSocket dla konkretnej trasy
async fn handle_route_socket(socket: WebSocket, route_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subskrybuj aktualizacje lokalizacji
    let mut vehicle_rx = state.vehicle_updates.subscribe();

    // Task do wysyłania aktualizacji (tylko dla danej trasy)
    let send_task = tokio::spawn(async move {
        loop {
            match vehicle_rx.recv().await {
                Ok(update) => {
                    // Filtruj tylko pojazdy z danej trasy
                    if update.route_id != Some(route_id) {
                        continue;
                    }

                    let msg = ServerMessage::VehicleLocation(update);
                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::error!("Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Task do odbierania wiadomości
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => recv_task.abort(),
        _ = recv_task => send_task.abort(),
    }
}

/// Obsługa połączenia WebSocket dla konkretnego pojazdu
async fn handle_vehicle_socket(socket: WebSocket, vehicle_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subskrybuj aktualizacje lokalizacji
    let mut vehicle_rx = state.vehicle_updates.subscribe();

    // Task do wysyłania aktualizacji (tylko dla danego pojazdu)
    let send_task = tokio::spawn(async move {
        loop {
            match vehicle_rx.recv().await {
                Ok(update) => {
                    // Filtruj tylko wybrany pojazd
                    if update.vehicle_id != vehicle_id {
                        continue;
                    }

                    let msg = ServerMessage::VehicleLocation(update);
                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::error!("Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Task do odbierania wiadomości
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => recv_task.abort(),
        _ = recv_task => send_task.abort(),
    }
}

/// Funkcja do broadcastowania aktualizacji lokalizacji
pub async fn broadcast_vehicle_location(
    sender: &VehicleUpdateSender,
    update: VehicleLocationUpdate,
) {
    if let Err(e) = sender.send(update) {
        tracing::warn!("Failed to broadcast vehicle location: {}", e);
    }
}

/// Funkcja do broadcastowania powiadomienia o incydencie
pub async fn broadcast_incident(
    sender: &VehicleUpdateSender,
    incident: IncidentWebSocketNotification,
) {
    // Tworzymy specjalny update z informacją o incydencie
    let update = VehicleLocationUpdate {
        vehicle_id: Uuid::nil(),
        registration_number: "INCIDENT".to_string(),
        route_id: Some(incident.route_id),
        route_number: Some(incident.route_number.clone()),
        route_color: None,
        driver_name: None,
        latitude: 0.0,
        longitude: 0.0,
        speed: None,
        heading: None,
        next_stop_id: None,
        next_stop_name: Some(incident.message_pl.clone()),
        eta_seconds: None,
        timestamp: incident.timestamp,
    };

    if let Err(e) = sender.send(update) {
        tracing::warn!("Failed to broadcast incident: {}", e);
    }
}

/// Inicjalizacja kanału broadcast dla aktualizacji pojazdów
pub fn init_vehicle_updates_channel() -> VehicleUpdateSender {
    let (tx, _rx) = broadcast::channel(1000);
    tx
}

/// Struktura do zarządzania aktywnymi połączeniami WebSocket
#[derive(Debug, Default)]
pub struct WebSocketManager {
    connections: Arc<tokio::sync::RwLock<Vec<Uuid>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn add_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.push(id);
        tracing::info!("WebSocket connection added. Total: {}", connections.len());
    }

    pub async fn remove_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.retain(|&conn_id| conn_id != id);
        tracing::info!("WebSocket connection removed. Total: {}", connections.len());
    }

    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_location_update_serialization() {
        let update = VehicleLocationUpdate {
            vehicle_id: Uuid::new_v4(),
            registration_number: "WX 12345".to_string(),
            route_id: Some(Uuid::new_v4()),
            route_number: Some("175".to_string()),
            route_color: Some("#2563EB".to_string()),
            driver_name: Some("Jan Kowalski".to_string()),
            latitude: 52.2297,
            longitude: 21.0122,
            speed: Some(45.5),
            heading: Some(180.0),
            next_stop_id: Some(Uuid::new_v4()),
            next_stop_name: Some("Dworzec Centralny".to_string()),
            eta_seconds: Some(120),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("WX 12345"));
        assert!(json.contains("52.2297"));
    }

    #[test]
    fn test_server_message_serialization() {
        let update = VehicleLocationUpdate {
            vehicle_id: Uuid::new_v4(),
            registration_number: "WX 12345".to_string(),
            route_id: None,
            route_number: None,
            route_color: None,
            driver_name: None,
            latitude: 52.2297,
            longitude: 21.0122,
            speed: None,
            heading: None,
            next_stop_id: None,
            next_stop_name: None,
            eta_seconds: None,
            timestamp: chrono::Utc::now(),
        };

        let msg = ServerMessage::VehicleLocation(update);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("vehicle_location"));
    }
}
