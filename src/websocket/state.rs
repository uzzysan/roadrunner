use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Typy klientów WebSocket
#[derive(Debug, Clone)]
pub enum ClientType {
    Driver { vehicle_id: Uuid },    // Kierowca - wysyła GPS
    Passenger,                       // Pasażer - odbiera GPS
    Parent { student_id: Uuid },    // Rodzic - śledzi dziecko
}

/// Klient WebSocket
#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub client_type: ClientType,
    pub subscribed_routes: Vec<Uuid>,  // Śledzone linie
    pub subscribed_vehicles: Vec<Uuid>, // Śledzone pojazdy
}

/// Stan WebSocket - zarządza wszystkimi klientami
pub struct WsState {
    clients: RwLock<HashMap<String, Client>>,
    // Kanał broadcast dla pozycji GPS
    gps_tx: broadcast::Sender<GpsBroadcast>,
}

#[derive(Debug, Clone)]
pub struct GpsBroadcast {
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
        let (gps_tx, _) = broadcast::channel(100);
        Self {
            clients: RwLock::new(HashMap::new()),
            gps_tx,
        }
    }
    
    /// Dodaj nowego klienta
    pub async fn add_client(&self) -> String {
        let client_id = Uuid::new_v4().to_string();
        let client = Client {
            id: client_id.clone(),
            client_type: ClientType::Passenger,
            subscribed_routes: vec![],
            subscribed_vehicles: vec![],
        };
        
        let mut clients = self.clients.write().await;
        clients.insert(client_id.clone(), client);
        
        client_id
    }
    
    /// Usuń klienta
    pub async fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
    }
    
    /// Zaktualizuj typ klienta
    pub async fn set_client_type(&self, client_id: &str, client_type: ClientType) -> Result<(), String> {
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
    
    /// Pobierz nadawcę broadcast GPS
    pub fn get_gps_sender(&self) -> broadcast::Sender<GpsBroadcast> {
        self.gps_tx.clone()
    }
    
    /// Pobierz odbiorcę broadcast GPS
    pub fn subscribe_gps(&self) -> broadcast::Receiver<GpsBroadcast> {
        self.gps_tx.subscribe()
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}
