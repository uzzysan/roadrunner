# RoadRunner — Szczegółowy Plan Rozwoju (dla agentów AI)

> **Projekt**: RoadRunner — System Transportu Zbiorowego i Szkolnego
> **Język**: Rust (Axum 0.7 + SQLx 0.7 + Tokio)
> **Baza danych**: PostgreSQL 16 + PostGIS
> **Mapy**: OpenStreetMap (darmowa, open-source)
> **Data planu**: 2026-03-26 (zaktualizowany)
> **Repozytorium**: `https://github.com/uzzysan/roadrunner`

---

## Spis treści

1. [Stan obecny — podsumowanie kodu](#1-stan-obecny)
2. [Faza 0 — Dokończenie infrastruktury](#faza-0) ✅
3. [Faza 1 — Auth v2 (zaawansowana autentykacja)](#faza-1) ✅
4. [Faza 2 — System biletowy QR](#faza-2) ✅
5. [Faza 3 — OpenStreetMap + Przystanki + Rozkłady](#faza-3) 🔄
6. [Faza 4 — GPS v2 (zaawansowany tracking)](#faza-4)
7. [Faza 5 — Moduł szkolny](#faza-5)
8. [Faza 6 — ETA i predykcja](#faza-6)
9. [Faza 7 — Admin API](#faza-7)
10. [Faza 8 — Mobile v2 (React Native)](#faza-8) ✅
11. [Faza 9 — Tauri Admin Panel](#faza-9)
12. [Faza 10 — Modularny system płatności (opcjonalny)](#faza-10)
13. [Faza 11 — RODO / Compliance](#faza-11)
14. [Faza 12 — Testy i optymalizacja](#faza-12)
15. [Konwencje i reguły kodu](#konwencje)

---

## Zmiany w planie (2026-03-26)

### 🔴 Usunięto
- ~~Faza 3 — Płatności Stripe (hardcoded)~~

### 🟢 Dodano
- **Faza 3** — OpenStreetMap + Przystanki + Rozkłady jazdy
- **Faza 10** — Modularny system płatności (opcjonalny, konfigurowalny)
  - Wsparcie dla wielu dostawców: Stripe, PayU, Tpay, etc.
  - Łatwa wymiana dostawcy przez konfigurację

---

## 1. Stan obecny — podsumowanie kodu <a id="1-stan-obecny"></a>

### ✅ Zakończone
- Faza 0: Infrastruktura (AppState, AppError, Design System)
- Faza 1: Auth v2 (MFA, middleware, role)
- Faza 2: System biletowy QR (generowanie, walidacja)
- Faza 8: Mobile setup (React Native, nawigacja, ekrany)

### 🔄 W trakcie
- Faza 3: OpenStreetMap + Przystanki + Rozkłady

---

## Faza 3 — OpenStreetMap + Przystanki + Rozkłady jazdy 🗺️ <a id="faza-3"></a>

**Cel**: Darmowa, open-source mapa z pełną funkcjonalnością transportową

### Backend (Rust)

#### 3.1 Model przystanku (`src/models/stop.rs`)
```rust
pub struct Stop {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location: Point, // PostGIS Point
    pub address: Option<String>,
    pub amenities: Vec<String>, // ["shelter", "bench", "timetable"]
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

pub struct StopWithDistance {
    pub stop: Stop,
    pub distance_meters: f64,
}
```

#### 3.2 Model rozkładu jazdy (`src/models/schedule.rs`)
```rust
pub struct Schedule {
    pub id: Uuid,
    pub route_id: Uuid,
    pub stop_id: Uuid,
    pub arrival_time: NaiveTime,
    pub departure_time: NaiveTime,
    pub day_type: DayType, // Weekday, Saturday, Sunday, Holiday
    pub is_active: bool,
}

pub enum DayType {
    Weekday,
    Saturday,
    Sunday,
    Holiday,
    Everyday,
}
```

#### 3.3 Model trasy (`src/models/route.rs`)
```rust
pub struct Route {
    pub id: Uuid,
    pub name: String, // "Linia 175"
    pub number: String, // "175"
    pub description: String,
    pub color: String, // HEX color
    pub stops: Vec<RouteStop>,
    pub is_active: bool,
}

pub struct RouteStop {
    pub stop_id: Uuid,
    pub order: i32,
    pub schedule: Vec<Schedule>,
}
```

#### 3.4 Handlery
- `GET /stops` — lista przystanków (z paginacją, filtrowaniem)
- `GET /stops/nearby?lat=...&lng=...&radius=...` — najbliższe przystanki
- `GET /stops/:id` — szczegóły przystanku
- `GET /stops/:id/schedules` — rozkład jazdy z przystanku
- `GET /routes` — lista tras
- `GET /routes/:id` — szczegóły trasy ze wszystkimi przystankami
- `GET /routes/:id/schedules` — pełny rozkład dla trasy
- `POST /stops/search` — wyszukiwanie przystanku po nazwie

#### 3.5 Integracja z OpenStreetMap (Nominatim)
- Geokodowanie adresów (adres → współrzędne)
- Reverse geokodowanie (współrzędne → adres)
- Wyszukiwanie POI (points of interest)

### Mobile (React Native)

#### 3.6 MapScreen z OpenStreetMap
```typescript
// Użycie react-native-maps z OpenStreetMap tiles
<MapView
  provider={PROVIDER_DEFAULT}
  customMapStyle={openStreetMapStyle}
  tileUrlTemplate="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
>
  {/* Markery przystanków */}
  {stops.map(stop => (
    <Marker
      key={stop.id}
      coordinate={{ latitude: stop.lat, longitude: stop.lng }}
      title={stop.name}
    />
  ))}

  {/* Trasa autobusu */}
  <Polyline
    coordinates={routeCoordinates}
    strokeColor={route.color}
    strokeWidth={4}
  />
</MapView>
```

#### 3.7 Funkcjonalności mapy
- 📍 Wyświetlanie przystanków jako markery
- 🚌 Wyświetlanie tras jako polilinie
- 🔍 Wyszukiwanie przystanku po nazwie
- 📍 Znajdź najbliższy przystanek (GPS)
- 🕐 Sprawdź rozkład jazdy (po kliknięciu w przystanek)
- 🚌 Pokaż trasę linii (po kliknięciu w linię)

#### 3.8 Ekrany
- **MapScreen** — mapa z przystankami i trasami
- **StopDetailsScreen** — szczegóły przystanku + rozkład
- **RouteDetailsScreen** — szczegóły trasy + wszystkie przystanki
- **SearchStopScreen** — wyszukiwanie przystanku

### Baza danych (PostGIS)

#### 3.9 Migracje
```sql
-- Przystanki z geometrią
CREATE TABLE stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location GEOGRAPHY(POINT, 4326) NOT NULL, -- PostGIS
    address VARCHAR(500),
    amenities TEXT[],
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indeks przestrzenny dla szybkich zapytań o odległość
CREATE INDEX idx_stops_location ON stops USING GIST(location);
CREATE INDEX idx_stops_active ON stops(is_active) WHERE is_active = true;

-- Rozkłady jazdy
CREATE TABLE schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID REFERENCES routes(id),
    stop_id UUID REFERENCES stops(id),
    arrival_time TIME NOT NULL,
    departure_time TIME NOT NULL,
    day_type day_type_enum NOT NULL,
    is_active BOOLEAN DEFAULT true
);

-- Trasy
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    number VARCHAR(20) NOT NULL UNIQUE,
    description TEXT,
    color VARCHAR(7) DEFAULT '#2563EB',
    is_active BOOLEAN DEFAULT true
);

-- Powiązanie tras z przystankami (kolejność)
CREATE TABLE route_stops (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID REFERENCES routes(id),
    stop_id UUID REFERENCES stops(id),
    stop_order INTEGER NOT NULL,
    UNIQUE(route_id, stop_order)
);
```

### Zalety OpenStreetMap
- ✅ **Darmowe** — brak limitów API, brak kluczy
- ✅ **Open-source** — można hostować własny tile server
- ✅ **Aktualne dane** — społeczność regularnie aktualizuje
- ✅ **Szczegółowe** — przystanki, nazwy ulic, budynki
- ✅ **Offline** — można pobrać tiles do aplikacji

---

## Faza 10 — Modularny system płatności (opcjonalny) 💳 <a id="faza-10"></a>

**Cel**: System płatności łatwy do dostosowania do różnych dostawców

### Architektura modułowa

```rust
// Trait dla dostawcy płatności
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_payment(&self, amount: i64, currency: &str) -> Result<PaymentIntent, Error>;
    async fn confirm_payment(&self, payment_id: &str) -> Result<PaymentStatus, Error>;
    async fn refund(&self, payment_id: &str, amount: Option<i64>) -> Result<(), Error>;
    fn name(&self) -> &str;
}

// Implementacje
pub struct StripeProvider { ... }
pub struct PayUProvider { ... }
pub struct TpayProvider { ... }

// Factory
pub fn create_provider(config: &Config) -> Box<dyn PaymentProvider> {
    match config.payment_provider.as_str() {
        "stripe" => Box::new(StripeProvider::new(config)),
        "payu" => Box::new(PayUProvider::new(config)),
        "tpay" => Box::new(TpayProvider::new(config)),
        _ => panic!("Unknown payment provider"),
    }
}
```

### Konfiguracja (`.env`)
```bash
# Wybór dostawcy: stripe, payu, tpay, none
PAYMENT_PROVIDER=stripe

# Stripe
STRIPE_SECRET_KEY=sk_...
STRIPE_WEBHOOK_SECRET=whsec_...

# PayU (opcjonalnie)
# PAYU_POS_ID=...
# PAYU_MD5_KEY=...

# Tpay (opcjonalnie)
# TPAY_ID=...
# TPAY_API_KEY=...
```

### Status
- 🟡 Podstawowa struktura gotowa (Faza 3 poprzednia)
- 🔴 Wymaga przetestowania z różnymi dostawcami
- 🔴 Wymaga dokumentacji dla klientów

---

## Zależności między fazami (zaktualizowane)

```
Faza 0 (Infrastruktura) ✅
    │
    ├──> Faza 1 (Auth v2) ✅
    │       │
    │       └──> Faza 2 (Bilety QR) ✅
    │               │
    │               ├──> Faza 3 (OpenStreetMap) 🔄
    │               │       │
    │               │       ├──> Faza 4 (GPS v2)
    │               │       └──> Faza 6 (ETA)
    │               │
    │               └──> Faza 10 (Płatności - opcjonalnie)
    │
    ├──> Faza 5 (Moduł szkolny)
    │
    └──> Faza 7 (Admin API)
            │
            └──> Faza 9 (Tauri Admin Panel)
```

---

## Konwencje kodu <a id="konwencje"></a>

### OpenStreetMap
- Używaj `tile.openstreetmap.org` dla developmentu
- Dla produkcji rozważ własny tile server lub innego providera (Mapbox, Carto)
- Zawsze podawaj attribution: "© OpenStreetMap contributors"

### PostGIS
- Używaj typu `GEOGRAPHY(POINT, 4326)` dla współrzędnych GPS
- Twórz indeksy GIST dla kolumn geometrii
- Używaj funkcji `ST_DWithin` dla zapytań o odległość
- Używaj `ST_Distance` z `::geography` dla dokładnych odległości w metrach

---

## Podsumowanie

| Faza | Opis | Status | Priorytet |
|------|------|--------|-----------|
| 0 | Infrastruktura | ✅ Done | Wysoki |
| 1 | Auth v2 | ✅ Done | Wysoki |
| 2 | Bilety QR | ✅ Done | Wysoki |
| 3 | OpenStreetMap + Przystanki | 🔄 In Progress | Wysoki |
| 4 | GPS v2 | 🔴 Todo | Średni |
| 5 | Moduł szkolny | 🔴 Todo | Średni |
| 6 | ETA i predykcja | 🔴 Todo | Średni |
| 7 | Admin API | 🔴 Todo | Średni |
| 8 | Mobile v2 | ✅ Done | Wysoki |
| 9 | Tauri Admin Panel | 🔴 Todo | Niski |
| 10 | Płatności (opcjonalne) | 🟡 Partial | Niski |
| 11 | RODO | 🔴 Todo | Średni |
| 12 | Testy i optymalizacja | 🔴 Todo | Wysoki |

---

**Ostatnia aktualizacja**: 2026-03-26
**Następna aktualizacja**: Po zakończeniu Fazy 3
