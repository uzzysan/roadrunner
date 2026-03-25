# RoadRunner - Plan Rozwoju Fazy 4: Zarządzanie Flotą

## 📋 Podsumowanie Fazy 4

Faza 4 wprowadza kompleksowy system zarządzania flotą pojazdów, kierowcami oraz zaawansowane funkcje dla transportu szkolnego.

---

## 🎯 Cele Fazy 4

1. **Zarządzanie pojazdami** - CRUD, statusy, przypisywanie do linii
2. **Zarządzanie kierowcami** - Profile, grafiki, uprawnienia
3. **GPS Tracking** - Real-time lokalizacja pojazdów na mapie
4. **System awarii** - Zgłaszanie, powiadomienia, pojazdy zastępcze
5. **School Transport** - Rejestracja dzieci, powiadomienia dla rodziców
6. **Panel administracyjny** - Dashboard, raporty, zarządzanie incydentami

---

## 🗄️ Modele Danych

### Vehicle (Pojazd)
```rust
pub struct Vehicle {
    pub id: Uuid,
    pub registration_number: String,  // Rejestracja (np. "WX 12345")
    pub vin: String,                  // Numer VIN
    pub brand: String,                // Marka (np. "Mercedes", "MAN")
    pub model: String,                // Model
    pub year: i32,                    // Rok produkcji
    pub capacity: i32,                // Liczba miejsc
    pub vehicle_type: VehicleType,    // Bus, Minibus, Coach
    pub fuel_type: FuelType,          // Diesel, Electric, Hybrid
    pub status: VehicleStatus,        // Active, Maintenance, Retired, Broken
    pub gps_device_id: Option<String>,// ID urządzenia GPS
    pub last_location: Option<String>,// "SRID=4326;POINT(lon lat)"
    pub last_location_at: Option<DateTime<Utc>>,
    pub current_driver_id: Option<Uuid>,
    pub current_route_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Driver (Kierowca)
```rust
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,                // Powiązanie z users
    pub employee_id: String,          // Numer pracownika
    pub license_number: String,       // Nr prawa jazdy
    pub license_categories: Vec<String>, // ["D", "DE"]
    pub license_expiry: NaiveDate,
    pub phone: String,
    pub emergency_contact: String,
    pub status: DriverStatus,         // Active, OnLeave, Suspended, Inactive
    pub assigned_vehicle_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### VehicleLocation (Historia lokalizacji - opcjonalnie)
```rust
pub struct VehicleLocation {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub location: String,             // "SRID=4326;POINT(lon lat)"
    pub speed: Option<f64>,           // km/h
    pub heading: Option<f64>,         // kierunek w stopniach
    pub recorded_at: DateTime<Utc>,
}
```

### Incident (Zdarzenie/Awaria)
```rust
pub struct Incident {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Uuid,
    pub incident_type: IncidentType,  // Breakdown, Accident, Delay, Other
    pub severity: Severity,           // Low, Medium, High, Critical
    pub title: String,
    pub description: String,
    pub location: Option<String>,     // Gdzie wystąpiło
    pub reported_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolution_notes: Option<String>,
    pub status: IncidentStatus,       // Reported, InProgress, Resolved, Cancelled
    pub replacement_vehicle_id: Option<Uuid>,
    pub estimated_resolution: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum IncidentType {
    Breakdown,        // Awaria techniczna
    Accident,         // Wypadek
    Delay,            // Opóźnienie
    RouteChange,      // Zmiana trasy
    Other,
}

pub enum Severity {
    Low,      // Nie wpływa na rozkład
    Medium,   // Lekkie opóźnienie
    High,     // Poważne opóźnienie
    Critical, // Pojazd wyłączony z ruchu
}
```

### IncidentNotification (Powiadomienie o incydencie)
```rust
pub struct IncidentNotification {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub route_id: Uuid,
    pub message_pl: String,
    pub message_en: String,
    pub sent_at: DateTime<Utc>,
    pub affected_users_count: i32,
}
```

### ChildRegistration (Rejestracja dziecka - School Transport)
```rust
pub struct ChildRegistration {
    pub id: Uuid,
    pub parent_user_id: Uuid,
    pub child_first_name: String,
    pub child_last_name: String,
    pub child_birth_date: NaiveDate,
    pub school_name: String,
    pub school_address: Option<String>,
    pub assigned_route_id: Uuid,
    pub pickup_stop_id: Uuid,
    pub dropoff_stop_id: Uuid,
    pub qr_code: String,              // Unikalny kod
    pub qr_code_data: String,         // Dane do wygenerowania QR
    pub photo_url: Option<String>,
    pub status: ChildStatus,          // Active, Inactive, Suspended
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ChildAttendance (Obecność dziecka)
```rust
pub struct ChildAttendance {
    pub id: Uuid,
    pub child_id: Uuid,
    pub route_id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Uuid,
    pub pickup_stop_id: Option<Uuid>,
    pub pickup_time: Option<DateTime<Utc>>,
    pub dropoff_stop_id: Option<Uuid>,
    pub dropoff_time: Option<DateTime<Utc>>,
    pub status: AttendanceStatus,     // Scheduled, PickedUp, DroppedOff, Absent, Cancelled
    pub confirmed_by: ConfirmationMethod, // QRCode, Manual, AutoGPS
    pub parent_notified_pickup: bool,
    pub parent_notified_dropoff: bool,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

pub enum AttendanceStatus {
    Scheduled,    // Zaplanowane
    PickedUp,     // Odebrane
    DroppedOff,   // Dostarczone
    Absent,       // Nieobecne
    Cancelled,    // Anulowane
}

pub enum ConfirmationMethod {
    QRCode,       // Skan kodu QR
    Manual,       // Ręczne potwierdzenie przez kierowcę
    AutoGPS,      // Automatycznie na podstawie GPS
}
```

### ParentNotification (Powiadomienie dla rodzica)
```rust
pub struct ParentNotification {
    pub id: Uuid,
    pub parent_user_id: Uuid,
    pub child_id: Option<Uuid>,
    pub notification_type: ParentNotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>, // Dodatkowe dane (np. location)
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub channel: NotificationChannel,   // Push, SMS, Email
}

pub enum ParentNotificationType {
    ChildPickedUp,      // Dziecko wsiadło
    ChildDroppedOff,    // Dziecko wysiadło
    BusDelayed,         // Autobus opóźniony
    BusApproaching,     // Autobus zbliża się do przystanku
    RouteChanged,       // Zmiana trasy
    Incident,           // Incydent
    General,            // Ogólne
}
```

---

## 🔌 Endpointy API

### Vehicles
```
GET    /vehicles              # Lista pojazdów
POST   /vehicles              # Dodaj pojazd
GET    /vehicles/:id          # Szczegóły pojazdu
PUT    /vehicles/:id          # Edytuj pojazd
DELETE /vehicles/:id          # Usuń pojazd
GET    /vehicles/:id/location # Aktualna lokalizacja
POST   /vehicles/:id/location # Aktualizuj lokalizację (z GPS)
GET    /vehicles/:id/history  # Historia lokalizacji
```

### Drivers
```
GET    /drivers               # Lista kierowców
POST   /drivers               # Dodaj kierowcę
GET    /drivers/:id           # Szczegóły kierowcy
PUT    /drivers/:id           # Edytuj kierowcę
DELETE /drivers/:id           # Usuń kierowcę
POST   /drivers/:id/assign    # Przypisz do pojazdu
POST   /drivers/:id/unassign  # Usuń przypisanie
```

### Incidents
```
GET    /incidents             # Lista incydentów
POST   /incidents             # Zgłoś incydent
GET    /incidents/:id         # Szczegóły incydentu
PUT    /incidents/:id         # Aktualizuj incydent
POST   /incidents/:id/resolve # Rozwiąż incydent
GET    /incidents/active      # Aktywne incydenty
```

### Child Registration (School Transport)
```
GET    /children                    # Lista dzieci (rodzic widzi swoje)
POST   /children                    # Zarejestruj dziecko
GET    /children/:id                # Szczegóły dziecka
PUT    /children/:id                # Edytuj dane dziecka
DELETE /children/:id                # Usuń rejestrację
GET    /children/:id/qr             # Pobierz kod QR
GET    /children/:id/attendance     # Historia obecności
```

### Child Attendance (Driver API)
```
POST   /attendance/scan             # Skanuj QR (wsiadanie/wysiadanie)
POST   /attendance/manual           # Manualne potwierdzenie
GET    /attendance/today            # Dzisiejsza lista (dla kierowcy)
GET    /attendance/:child_id/status # Status dziecka
```

### Parent Notifications
```
GET    /notifications               # Lista powiadomień
POST   /notifications/:id/read      # Oznacz jako przeczytane
GET    /notifications/unread        # Nieprzeczytane
PUT    /notifications/settings      # Ustawienia powiadomień
```

### Real-time (WebSocket)
```
WS     /ws/vehicles                 # Stream lokalizacji pojazdów
WS     /ws/route/:id                # Stream dla konkretnej trasy
```

---

## 📱 Ekrany Mobile

### Dla Kierowcy
1. **DriverDashboard** - Aktualna trasa, status, przyciski akcji
2. **IncidentReport** - Formularz zgłaszania awarii
3. **ChildrenList** - Lista dzieci do odebrania (school transport)
4. **QRScanner** - Skanowanie kodów QR dzieci
5. **RouteStatus** - Status trasy, opóźnienia

### Dla Rodzica
1. **MyChildren** - Lista zarejestrowanych dzieci
2. **ChildDetails** - Szczegóły dziecka, trasa, QR code
3. **LiveTracking** - Śledzenie autobusu na żywo
4. **AttendanceHistory** - Historia przejazdów
5. **Notifications** - Powiadomienia

### Dla Pasażera (Public Transport)
1. **LiveVehicles** - Pojazdy na trasie na żywo
2. **ETA Display** - Szacowany czas przyjazdu

---

## 🎛️ Panel Administracyjny (Web)

### Dashboard
- Mapa z wszystkimi pojazdami na żywo
- Lista aktywnych incydentów
- Statystyki dnia (aktywne pojazdy, opóźnienia)

### Zarządzanie Flotą
- Lista pojazdów z filtrowaniem
- Formularz dodawania/edycji pojazdu
- Historia serwisowa

### Zarządzanie Kierowcami
- Lista kierowców
- Grafiki
- Uprawnienia i certyfikaty

### Incydenty
- Lista zgłoszeń
- Szczegóły incydentu
- Przypisywanie pojazdów zastępczych
- Komunikaty dla pasażerów

### School Transport
- Lista zarejestrowanych dzieci
- Raporty obecności
- Zarządzanie trasami szkolnymi

---

## 🔄 WebSocket - Real-time Updates

### Wysyłane przez serwer:
```json
// vehicle_location_update
{
  "type": "vehicle_location",
  "vehicle_id": "uuid",
  "route_id": "uuid",
  "location": {"lat": 52.2297, "lon": 21.0122},
  "speed": 45.5,
  "heading": 180,
  "next_stop_id": "uuid",
  "eta_seconds": 120,
  "timestamp": "2024-01-15T10:30:00Z"
}

// incident_notification
{
  "type": "incident",
  "incident_id": "uuid",
  "route_id": "uuid",
  "severity": "high",
  "message_pl": "Opóźnienie linii 175 o 15 minut",
  "message_en": "Line 175 delayed by 15 minutes",
  "timestamp": "2024-01-15T10:30:00Z"
}

// child_pickup_notification (do rodzica)
{
  "type": "child_picked_up",
  "child_id": "uuid",
  "pickup_stop": "Dworzec Centralny",
  "pickup_time": "2024-01-15T07:45:00Z",
  "vehicle_id": "uuid"
}
```

---

## 📊 Migracje Bazy Danych

### 0007_vehicles_drivers.sql
```sql
-- Vehicle types
CREATE TYPE vehicle_type AS ENUM ('bus', 'minibus', 'coach', 'tram', 'trolleybus');
CREATE TYPE fuel_type AS ENUM ('diesel', 'electric', 'hybrid', 'cng', 'hydrogen');
CREATE TYPE vehicle_status AS ENUM ('active', 'maintenance', 'retired', 'broken');
CREATE TYPE driver_status AS ENUM ('active', 'on_leave', 'suspended', 'inactive');

-- Vehicles table
CREATE TABLE vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registration_number VARCHAR(20) UNIQUE NOT NULL,
    vin VARCHAR(17) UNIQUE,
    brand VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    year INTEGER,
    capacity INTEGER NOT NULL DEFAULT 50,
    vehicle_type vehicle_type NOT NULL DEFAULT 'bus',
    fuel_type fuel_type NOT NULL DEFAULT 'diesel',
    status vehicle_status NOT NULL DEFAULT 'active',
    gps_device_id VARCHAR(100),
    last_location GEOGRAPHY(POINT, 4326),
    last_location_at TIMESTAMPTZ,
    current_driver_id UUID REFERENCES users(id),
    current_route_id UUID REFERENCES routes(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_vehicles_location ON vehicles USING GIST(last_location);
CREATE INDEX idx_vehicles_status ON vehicles(status);
CREATE INDEX idx_vehicles_route ON vehicles(current_route_id);

-- Drivers table
CREATE TABLE drivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    employee_id VARCHAR(50) UNIQUE,
    license_number VARCHAR(50) NOT NULL,
    license_categories TEXT[] NOT NULL DEFAULT '{}',
    license_expiry DATE NOT NULL,
    phone VARCHAR(20) NOT NULL,
    emergency_contact VARCHAR(100),
    status driver_status NOT NULL DEFAULT 'active',
    assigned_vehicle_id UUID REFERENCES vehicles(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE INDEX idx_drivers_user ON drivers(user_id);
CREATE INDEX idx_drivers_status ON drivers(status);
```

### 0008_incidents.sql
```sql
-- Incident types
CREATE TYPE incident_type AS ENUM ('breakdown', 'accident', 'delay', 'route_change', 'other');
CREATE TYPE incident_severity AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE incident_status AS ENUM ('reported', 'in_progress', 'resolved', 'cancelled');

-- Incidents table
CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id),
    driver_id UUID NOT NULL REFERENCES drivers(id),
    incident_type incident_type NOT NULL,
    severity incident_severity NOT NULL DEFAULT 'medium',
    title VARCHAR(255) NOT NULL,
    description TEXT,
    location GEOGRAPHY(POINT, 4326),
    reported_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    resolution_notes TEXT,
    status incident_status NOT NULL DEFAULT 'reported',
    replacement_vehicle_id UUID REFERENCES vehicles(id),
    estimated_resolution TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_incidents_vehicle ON incidents(vehicle_id);
CREATE INDEX idx_incidents_status ON incidents(status);
CREATE INDEX idx_incidents_type ON incidents(incident_type);

-- Incident notifications
CREATE TABLE incident_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id),
    route_id UUID NOT NULL REFERENCES routes(id),
    message_pl TEXT NOT NULL,
    message_en TEXT NOT NULL,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    affected_users_count INTEGER DEFAULT 0
);
```

### 0009_school_transport.sql
```sql
-- Child status
CREATE TYPE child_status AS ENUM ('active', 'inactive', 'suspended');
CREATE TYPE attendance_status AS ENUM ('scheduled', 'picked_up', 'dropped_off', 'absent', 'cancelled');
CREATE TYPE confirmation_method AS ENUM ('qr_code', 'manual', 'auto_gps');
CREATE TYPE parent_notification_type AS ENUM (
    'child_picked_up', 'child_dropped_off', 'bus_delayed', 
    'bus_approaching', 'route_changed', 'incident', 'general'
);
CREATE TYPE notification_channel AS ENUM ('push', 'sms', 'email');

-- Child registrations
CREATE TABLE child_registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_user_id UUID NOT NULL REFERENCES users(id),
    child_first_name VARCHAR(100) NOT NULL,
    child_last_name VARCHAR(100) NOT NULL,
    child_birth_date DATE NOT NULL,
    school_name VARCHAR(255) NOT NULL,
    school_address TEXT,
    assigned_route_id UUID REFERENCES routes(id),
    pickup_stop_id UUID REFERENCES stops(id),
    dropoff_stop_id UUID REFERENCES stops(id),
    qr_code VARCHAR(255) UNIQUE NOT NULL,
    qr_code_data TEXT NOT NULL,
    photo_url TEXT,
    status child_status NOT NULL DEFAULT 'active',
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_children_parent ON child_registrations(parent_user_id);
CREATE INDEX idx_children_route ON child_registrations(assigned_route_id);
CREATE INDEX idx_children_qr ON child_registrations(qr_code);

-- Child attendance
CREATE TABLE child_attendance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    child_id UUID NOT NULL REFERENCES child_registrations(id),
    route_id UUID NOT NULL REFERENCES routes(id),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id),
    driver_id UUID NOT NULL REFERENCES drivers(id),
    pickup_stop_id UUID REFERENCES stops(id),
    pickup_time TIMESTAMPTZ,
    dropoff_stop_id UUID REFERENCES stops(id),
    dropoff_time TIMESTAMPTZ,
    status attendance_status NOT NULL DEFAULT 'scheduled',
    confirmed_by confirmation_method,
    parent_notified_pickup BOOLEAN DEFAULT FALSE,
    parent_notified_dropoff BOOLEAN DEFAULT FALSE,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_attendance_child ON child_attendance(child_id);
CREATE INDEX idx_attendance_date ON child_attendance(date);
CREATE INDEX idx_attendance_route ON child_attendance(route_id);

-- Parent notifications
CREATE TABLE parent_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_user_id UUID NOT NULL REFERENCES users(id),
    child_id UUID REFERENCES child_registrations(id),
    notification_type parent_notification_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    read_at TIMESTAMPTZ,
    channel notification_channel NOT NULL DEFAULT 'push'
);

CREATE INDEX idx_notifications_parent ON parent_notifications(parent_user_id);
CREATE INDEX idx_notifications_unread ON parent_notifications(parent_user_id, read_at) WHERE read_at IS NULL;
```

---

## 📅 Plan Implementacji

### Sprint 1: Podstawy Floty (2-3 dni)
- [ ] Migracje: vehicles, drivers
- [ ] Modele: Vehicle, Driver
- [ ] Handlery: vehicle CRUD, driver CRUD
- [ ] Mobile: VehicleList, DriverList (admin)

### Sprint 2: GPS Tracking (2-3 dni)
- [ ] GPS endpointy
- [ ] WebSocket server
- [ ] Mobile: Live tracking map
- [ ] Historia lokalizacji

### Sprint 3: System Awarii (2-3 dni)
- [ ] Migracja: incidents
- [ ] Modele: Incident
- [ ] Handlery: incident CRUD
- [ ] Mobile: IncidentReport (driver)
- [ ] Powiadomienia o opóźnieniach

### Sprint 4: Pojazdy Zastępcze (1-2 dni)
- [ ] Logika przypisywania zastępczych
- [ ] Panel administracyjny
- [ ] Aktualizacja rozkładów

### Sprint 5: School Transport (3-4 dni)
- [ ] Migracje: child_registrations, child_attendance
- [ ] Modele: ChildRegistration, ChildAttendance
- [ ] QR code generation
- [ ] Mobile: ChildrenList, QRScanner (driver)
- [ ] Mobile: MyChildren, ChildDetails (parent)

### Sprint 6: Powiadomienia (2-3 dni)
- [ ] Parent notifications system
- [ ] Push notifications (Firebase)
- [ ] Mobile: Notifications screen
- [ ] Ustawienia powiadomień

### Sprint 7: Panel Admina (2-3 dni)
- [ ] Admin dashboard
- [ ] Fleet management
- [ ] Incident management
- [ ] Reports

---

## 🚀 Uruchomienie WebSocket Server

```rust
// Przykład integracji z Axum
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Subscribe to vehicle location updates
    let mut rx = state.vehicle_updates.subscribe();
    
    while let Ok(update) = rx.recv().await {
        let msg = serde_json::to_string(&update).unwrap();
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
```

---

## 📊 Szacowane Zasoby

| Komponent | Szacowany czas | Priorytet |
|-----------|---------------|-----------|
| Podstawy floty | 2-3 dni | Wysoki |
| GPS Tracking | 2-3 dni | Wysoki |
| System awarii | 2-3 dni | Wysoki |
| Pojazdy zastępcze | 1-2 dni | Średni |
| School Transport | 3-4 dni | Wysoki |
| Powiadomienia | 2-3 dni | Wysoki |
| Panel admina | 2-3 dni | Średni |
| **RAZEM** | **14-21 dni** | - |

---

## ⚠️ Ryzyka i Wyzwania

1. **Skalowalność WebSocket** - Duża liczba równoczesnych połączeń
2. **Bateria GPS** - Częste aktualizacje zużywają baterię
3. **Dokładność GPS** - W budynkach/miastach może być niedokładne
4. **GDPR** - Dane dzieci wymagają szczególnej ochrony
5. **Offline mode** - Kierowca może nie mieć internetu

---

**Status:** 🔄 Gotowy do implementacji  
**Ostatnia aktualizacja:** 2024-01-15
