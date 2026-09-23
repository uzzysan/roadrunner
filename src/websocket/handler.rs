use axum::extract::ws::{Message, WebSocket};
use uuid::Uuid;

use crate::models::user::UserRole;
use crate::state::AppState;
use crate::tenant::TenantIdentity;
use crate::websocket::state::{ClientType, GpsBroadcast};

/// Przetwórz wiadomość od klienta
pub async fn process_message(
    client_id: &str,
    text: &str,
    state: &AppState,
    identity: &TenantIdentity,
    socket: &mut WebSocket,
) -> Result<(), String> {
    // Parsuj JSON
    let msg: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {}", e))?;

    let msg_type = msg
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or("Missing message type")?;

    match msg_type {
        // Kierowca: autentykacja
        "auth_driver" => {
            let claims = identity
                .claims
                .as_ref()
                .ok_or("Driver authentication requires a valid access token")?;
            if !matches!(&claims.role, UserRole::Driver) {
                return Err("Driver role required".to_string());
            }

            let vehicle_id = msg
                .get("vehicle_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing vehicle_id")?;
            let vehicle_uuid = Uuid::parse_str(vehicle_id).map_err(|_| "Invalid vehicle_id")?;

            let is_assigned: bool = sqlx::query_scalar(
                "SELECT EXISTS(\
                    SELECT 1 FROM drivers \
                    WHERE user_id = $1 AND assigned_vehicle_id = $2 AND status = 'active'\
                )",
            )
            .bind(claims.sub)
            .bind(vehicle_uuid)
            .fetch_one(&state.db)
            .await
            .map_err(|error| format!("Driver authorization failed: {error}"))?;
            if !is_assigned {
                return Err("Driver is not assigned to this vehicle".to_string());
            }

            state
                .ws
                .set_client_type(
                    client_id,
                    ClientType::Driver {
                        vehicle_id: vehicle_uuid,
                    },
                )
                .await
                .map_err(|e| e.to_string())?;

            let response = r#"{"type":"auth_success","role":"driver"}"#;
            let _ = socket.send(Message::Text(response.to_string())).await;
            Ok(())
        }

        // Pasażer: autentykacja
        "auth_passenger" => {
            state
                .ws
                .set_client_type(client_id, ClientType::Passenger)
                .await
                .map_err(|e| e.to_string())?;

            let response = r#"{"type":"auth_success","role":"passenger"}"#;
            let _ = socket.send(Message::Text(response.to_string())).await;
            Ok(())
        }

        // Subskrypcja linii
        "subscribe_route" => {
            let route_id = msg
                .get("route_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing route_id")?;
            let route_uuid = Uuid::parse_str(route_id).map_err(|_| "Invalid route_id")?;

            let route_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM routes WHERE id = $1)")
                    .bind(route_uuid)
                    .fetch_one(&state.db)
                    .await
                    .map_err(|error| format!("Route authorization failed: {error}"))?;
            if !route_exists {
                return Err("Route is not available for this carrier".to_string());
            }

            state
                .ws
                .subscribe_route(client_id, route_uuid)
                .await
                .map_err(|e| e.to_string())?;

            let response = format!(r#"{{"type":"subscribed","route_id":"{}"}}"#, route_id);
            let _ = socket.send(Message::Text(response)).await;
            Ok(())
        }

        // Subskrypcja pojazdu
        "subscribe_vehicle" => {
            let vehicle_id = msg
                .get("vehicle_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing vehicle_id")?;
            let vehicle_uuid = Uuid::parse_str(vehicle_id).map_err(|_| "Invalid vehicle_id")?;

            let vehicle_exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM vehicles WHERE id = $1)")
                    .bind(vehicle_uuid)
                    .fetch_one(&state.db)
                    .await
                    .map_err(|error| format!("Vehicle authorization failed: {error}"))?;
            if !vehicle_exists {
                return Err("Vehicle is not available for this carrier".to_string());
            }

            state
                .ws
                .subscribe_vehicle(client_id, vehicle_uuid)
                .await
                .map_err(|e| e.to_string())?;

            let response = format!(r#"{{"type":"subscribed","vehicle_id":"{}"}}"#, vehicle_id);
            let _ = socket.send(Message::Text(response)).await;
            Ok(())
        }

        // Kierowca: wysyłka pozycji GPS
        "gps_update" => {
            let vehicle_id = msg
                .get("vehicle_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing vehicle_id")?;
            let vehicle_uuid = Uuid::parse_str(vehicle_id).map_err(|_| "Invalid vehicle_id")?;

            let latitude = msg
                .get("latitude")
                .and_then(|v| v.as_f64())
                .ok_or("Missing latitude")?;
            let longitude = msg
                .get("longitude")
                .and_then(|v| v.as_f64())
                .ok_or("Missing longitude")?;
            let speed_kmh = msg.get("speed_kmh").and_then(|v| v.as_f64());
            let heading = msg
                .get("heading")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);

            if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
                return Err("Invalid GPS coordinates".to_string());
            }

            let client = state.ws.client(client_id).await?;
            if client.carrier_id != identity.carrier_id
                || !matches!(
                    client.client_type,
                    ClientType::Driver { vehicle_id } if vehicle_id == vehicle_uuid
                )
            {
                return Err("Driver is not authenticated for this vehicle".to_string());
            }

            let route_id: Option<Uuid> =
                sqlx::query_scalar("SELECT current_route_id FROM vehicles WHERE id = $1")
                    .bind(vehicle_uuid)
                    .fetch_optional(&state.db)
                    .await
                    .map_err(|error| format!("Vehicle lookup failed: {error}"))?
                    .flatten();

            sqlx::query(
                "INSERT INTO vehicle_locations (vehicle_id, location, speed, heading) \
                 VALUES ($1, ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography, $4, $5)",
            )
            .bind(vehicle_uuid)
            .bind(longitude)
            .bind(latitude)
            .bind(speed_kmh)
            .bind(heading.map(f64::from))
            .execute(&state.db)
            .await
            .map_err(|error| format!("GPS update failed: {error}"))?;

            // Broadcast only inside this carrier's dedicated channel.
            let broadcast = GpsBroadcast {
                carrier_id: identity.carrier_id,
                vehicle_id: vehicle_uuid,
                latitude,
                longitude,
                speed_kmh,
                heading,
                route_id,
                next_stop: None,
                next_stop_eta: None,
                timestamp: chrono::Utc::now(),
            };

            let _ = state
                .ws
                .get_gps_sender(identity.carrier_id)
                .await
                .send(broadcast);

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
