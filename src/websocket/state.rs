use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Typy klientów WebSocket
#[derive(Debug, Clone)]
pub enum ClientType {
    Driver { vehicle_id: Uuid }, // Kierowca - wysyła GPS
    Passenger,                   // Pasażer - odbiera GPS
    Parent { student_id: Uuid }, // Rodzic - śledzi dziecko
}

/// Klient WebSocket
#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub carrier_id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_type: ClientType,
    pub subscribed_routes: Vec<Uuid>,   // Śledzone linie
    pub subscribed_vehicles: Vec<Uuid>, // Śledzone pojazdy
}

/// Stan WebSocket - zarządza wszystkimi klientami
pub struct WsState {
    clients: RwLock<HashMap<String, Client>>,
    // A separate channel per carrier makes cross-tenant delivery impossible
    // even if route or vehicle UUIDs are ever reused by an importer.
    gps_channels: RwLock<HashMap<Uuid, broadcast::Sender<GpsBroadcast>>>,
}

#[derive(Debug, Clone)]
pub struct GpsBroadcast {
    pub carrier_id: Uuid,
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: Option<f64>,
    pub heading: Option<i32>,
    pub route_id: Option<Uuid>,
    pub next_stop: Option<String>,
    pub next_stop_eta: Option<i32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WsState {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            gps_channels: RwLock::new(HashMap::new()),
        }
    }

    /// Dodaj nowego klienta
    pub async fn add_client(&self, carrier_id: Uuid, user_id: Option<Uuid>) -> String {
        let client_id = Uuid::new_v4().to_string();
        let client = Client {
            id: client_id.clone(),
            carrier_id,
            user_id,
            client_type: ClientType::Passenger,
            subscribed_routes: vec![],
            subscribed_vehicles: vec![],
        };

        let mut clients = self.clients.write().await;
        clients.insert(client_id.clone(), client);

        client_id
    }

    pub async fn client(&self, client_id: &str) -> Result<Client, String> {
        self.clients
            .read()
            .await
            .get(client_id)
            .cloned()
            .ok_or_else(|| "Client not found".to_string())
    }

    /// Usuń klienta
    pub async fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
    }

    /// Zaktualizuj typ klienta
    pub async fn set_client_type(
        &self,
        client_id: &str,
        client_type: ClientType,
    ) -> Result<(), String> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.client_type = client_type;
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    /// Subskrybuj linię
    pub async fn subscribe_route(&self, client_id: &str, route_id: Uuid) -> Result<(), String> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            if !client.subscribed_routes.contains(&route_id) {
                client.subscribed_routes.push(route_id);
            }
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    /// Subskrybuj pojazd
    pub async fn subscribe_vehicle(&self, client_id: &str, vehicle_id: Uuid) -> Result<(), String> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            if !client.subscribed_vehicles.contains(&vehicle_id) {
                client.subscribed_vehicles.push(vehicle_id);
            }
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    /// Pobierz nadawcę broadcast GPS dla jednego przewoźnika.
    pub async fn get_gps_sender(&self, carrier_id: Uuid) -> broadcast::Sender<GpsBroadcast> {
        let mut channels = self.gps_channels.write().await;
        channels
            .entry(carrier_id)
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    }

    /// Pobierz odbiorcę broadcast GPS dla jednego przewoźnika.
    pub async fn subscribe_gps(&self, carrier_id: Uuid) -> broadcast::Receiver<GpsBroadcast> {
        self.get_gps_sender(carrier_id).await.subscribe()
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{GpsBroadcast, WsState};
    use tokio::sync::broadcast::error::TryRecvError;
    use uuid::Uuid;

    fn update(carrier_id: Uuid) -> GpsBroadcast {
        GpsBroadcast {
            carrier_id,
            vehicle_id: Uuid::new_v4(),
            latitude: 52.2297,
            longitude: 21.0122,
            speed_kmh: Some(35.0),
            heading: Some(90),
            route_id: None,
            next_stop: None,
            next_stop_eta: None,
            timestamp: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn gps_channels_and_clients_are_carrier_scoped() {
        let state = WsState::new();
        let carrier_a = Uuid::new_v4();
        let carrier_b = Uuid::new_v4();
        let user_a = Uuid::new_v4();

        let client_id = state.add_client(carrier_a, Some(user_a)).await;
        let client = state.client(&client_id).await.expect("client exists");
        assert_eq!(client.carrier_id, carrier_a);
        assert_eq!(client.user_id, Some(user_a));

        let mut receiver_a = state.subscribe_gps(carrier_a).await;
        let mut receiver_b = state.subscribe_gps(carrier_b).await;
        let message = update(carrier_a);
        state
            .get_gps_sender(carrier_a)
            .await
            .send(message.clone())
            .expect("carrier A has a receiver");

        let received = receiver_a.recv().await.expect("carrier A receives update");
        assert_eq!(received.carrier_id, carrier_a);
        assert_eq!(received.vehicle_id, message.vehicle_id);
        assert!(matches!(receiver_b.try_recv(), Err(TryRecvError::Empty)));
    }
}
