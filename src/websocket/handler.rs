use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use uuid::Uuid;

use crate::websocket::state::{WsState, ClientType, GpsBroadcast};

/// Przetwórz wiadomość od klienta
pub async fn process_message(
    client_id: &str,
    text: &str,
    state: &Arc<WsState>,
    socket: &mut WebSocket,
) -> Result<(), String> {
    // Parsuj JSON
    let msg: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    let msg_type = msg.get("type")
        .and_then(|v| v.as_str())
        .ok_or("Missing message type")?;
    
    match msg_type {
        // Kierowca: autentykacja
        "auth_driver" => {
            let vehicle_id = msg.get("vehicle_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing vehicle_id")?;
            let vehicle_uuid = Uuid::parse_str(vehicle_id)
                .map_err(|_| "Invalid vehicle_id")?;
            
            state.set_client_type(
                client_id, 
                ClientType::Driver { vehicle_id: vehicle_uuid }
            ).await.map_err(|e| e.to_string())?;
            
            let response = r#"{"type":"auth_success","role":"driver"}"#;
            let _ = socket.send(Message::Text(response.to_string())).await;
            Ok(())
        }
        
        // Pasażer: autentykacja
        "auth_passenger" => {
            state.set_client_type(client_id, ClientType::Passenger)
                .await.map_err(|e| e.to_string())?;
            
            let response = r#"{"type":"auth_success","role":"passenger"}"#;
            let _ = socket.send(Message::Text(response.to_string())).await;
            Ok(())
        }
        
        // Subskrypcja linii
        "subscribe_route" => {
            let route_id = msg.get("route_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing route_id")?;
            let route_uuid = Uuid::parse_str(route_id)
                .map_err(|_| "Invalid route_id")?;
            
            state.subscribe_route(client_id, route_uuid)
                .await.map_err(|e| e.to_string())?;
            
            let response = format!(r#"{{"type":"subscribed","route_id":"{}"}}"#, route_id);
            let _ = socket.send(Message::Text(response)).await;
            Ok(())
        }
        
        // Subskrypcja pojazdu
        "subscribe_vehicle" => {
            let vehicle_id = msg.get("vehicle_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing vehicle_id")?;
            let vehicle_uuid = Uuid::parse_str(vehicle_id)
                .map_err(|_| "Invalid vehicle_id")?;
            
            state.subscribe_vehicle(client_id, vehicle_uuid)
                .await.map_err(|e| e.to_string())?;
            
            let response = format!(r#"{{"type":"subscribed","vehicle_id":"{}"}}"#, vehicle_id);
            let _ = socket.send(Message::Text(response)).await;
            Ok(())
        }
        
        // Kierowca: wysyłka pozycji GPS
        "gps_update" => {
            let vehicle_id = msg.get("vehicle_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing vehicle_id")?;
            let vehicle_uuid = Uuid::parse_str(vehicle_id)
                .map_err(|_| "Invalid vehicle_id")?;
            
            let latitude = msg.get("latitude")
                .and_then(|v| v.as_f64())
                .ok_or("Missing latitude")?;
            let longitude = msg.get("longitude")
                .and_then(|v| v.as_f64())
                .ok_or("Missing longitude")?;
            let speed_kmh = msg.get("speed_kmh").and_then(|v| v.as_f64());
            let heading = msg.get("heading").and_then(|v| v.as_i64()).map(|v| v as i32);
            
            // Broadcast do wszystkich subskrybentów
            let broadcast = GpsBroadcast {
                vehicle_id: vehicle_uuid,
                latitude,
                longitude,
                speed_kmh,
                heading,
                route_id: None, // TODO: pobierz z assignment
                next_stop: None,
                next_stop_eta: None,
                timestamp: chrono::Utc::now(),
            };
            
            let _ = state.get_gps_sender().send(broadcast);
            
            // Potwierdzenie dla kierowcy
            let response = r#"{"type":"gps_received"}"#;
            let _ = socket.send(Message::Text(response.to_string())).await;
            Ok(())
        }
        
        // Ping/Pong (heartbeat)
        "ping" => {
            let response = r#"{"type":"pong"}"#;
            let _ = socket.send(Message::Text(response.to_string())).await;
            Ok(())
        }
        
        _ => Err(format!("Unknown message type: {}", msg_type)),
    }
}
