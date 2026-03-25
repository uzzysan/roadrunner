# RoadRunner — Szczegółowy Plan Rozwoju (dla agentów AI)

> **Projekt**: RoadRunner — System Transportu Zbiorowego i Szkolnego
> **Język**: Rust (Axum 0.7 + SQLx 0.7 + Tokio)
> **Baza danych**: PostgreSQL 16 + PostGIS
> **Data planu**: 2026-03-25
> **Repozytorium**: `https://github.com/uzzysan/roadrunner`
> **Lokalizacja lokalna**: `/home/uzzy/Kodzenie/roadrunner`

---

## Spis treści

1. [Stan obecny — podsumowanie kodu](#1-stan-obecny)
2. [Faza 0 — Dokończenie infrastruktury](#faza-0)
3. [Faza 1 — Auth v2 (zaawansowana autentykacja)](#faza-1)
4. [Faza 2 — System biletowy QR](#faza-2)
5. [Faza 3 — Płatności Stripe](#faza-3)
6. [Faza 4 — GPS v2 (zaawansowany tracking)](#faza-4)
7. [Faza 5 — Moduł szkolny](#faza-5)
8. [Faza 6 — ETA i predykcja](#faza-6)
9. [Faza 7 — Admin API](#faza-7)
10. [Faza 8 — Mobile v2 (React Native)](#faza-8)
11. [Faza 9 — Tauri Admin Panel](#faza-9)
12. [Faza 10 — RODO / Compliance](#faza-10)
13. [Faza 11 — Testy i optymalizacja](#faza-11)
14. [Konwencje i reguły kodu](#konwencje)
15. [Mapowanie zależności między fazami](#zaleznosci)

---

## 1. Stan obecny — podsumowanie kodu <a id="1-stan-obecny"></a>

### Istniejące pliki źródłowe

| Ścieżka | Opis | Kompletność |
|----------|------|-------------|
| `src/main.rs` | Serwer Axum: 4 endpointy (`/`, `/health`, `/ws`, `/auth/*`) | Szkielet — wymaga rozbudowy routingu |
| `src/config.rs` | Struct `Config` z `from_env()` — `DATABASE_URL`, `JWT_SECRET`, `JWT_EXPIRATION`, `PORT`, `HOST` | Kompletny — dodać `REDIS_URL`, `STRIPE_*`, `MFA_*` |
| `src/lib.rs` | Re-export modułów: `auth`, `config`, `handlers`, `models`, `websocket` | Automatycznie rozbudowywać |
| `src/auth/jwt.rs` | `Claims`, `TokenPair`, `generate_token_pair()` — hardcoded secret! | Wymaga: env secret, `decode_token()`, middleware |
| `src/auth/password.rs` | `hash_password()`, `verify_password()` — bcrypt | Kompletny |
| `src/auth/mod.rs` | Re-export `jwt`, `password` | Dodać `middleware.rs`, `mfa.rs` |
| `src/handlers/auth.rs` | `register()`, `login()` — role hardcoded na `Passenger` | Wymaga: MFA, refresh token, role z DB |
| `src/handlers/mod.rs` | Re-export `auth` | Dodać: `routes`, `stops`, `vehicles`, `tickets`, `gps`, `students`, `admin` |
| `src/models/user.rs` | `User`, `UserRole`, `CreateUserRequest`, `LoginRequest`, `UserResponse` | Kompletny — dodać `UpdateUserRequest` |
| `src/models/student.rs` | `Student`, `ParentStudentLink`, `RelationshipType`, `StudentWithParents`, `ParentInfo` | Kompletny — dodać CRUD requests |
| `src/models/stop.rs` | `Stop`, `CreateStopRequest`, `StopResponse` | Kompletny |
| `src/models/route.rs` | `Route`, `RouteStop`, `RouteType`, `CreateRouteRequest`, `RouteStopInput` | Kompletny |
| `src/models/vehicle.rs` | `Vehicle`, `VehicleAssignment`, `CreateVehicleRequest`, `CreateAssignmentRequest` | Kompletny |
| `src/models/gps.rs` | `GpsPosition`, `GpsUpdateRequest`, `GpsPositionResponse`, `VehicleLocation` | Kompletny |
| `src/websocket/mod.rs` | `ws_handler()`, `handle_socket()` — obsługa WS z dispatch | Wymaga: JWT auth przy połączeniu, GPS storage |
| `src/websocket/handler.rs` | `process_message()` — dispatch: `auth_driver`, `auth_passenger`, `subscribe_route`, `subscribe_vehicle`, `gps_update`, `ping` | Wymaga: zapis GPS do DB, filtr Kalmana, ETA |
| `src/websocket/state.rs` | `WsState`, `Client`, `ClientType`, `GpsBroadcast` — tokio broadcast | Kompletny |

### Istniejące migracje

| Plik | Zawartość |
|------|-----------|
| `0001_initial_schema.sql` | Tabele: `users`, `students`, `parent_student_links` + typy enum + indeksy |
| `0002_transport_schema.sql` | Tabele: `stops`, `routes`, `route_stops`, `vehicles`, `vehicle_assignments`, `gps_positions` + indeksy GIST |

### Kluczowe braki w istniejącym kodzie

1. **JWT secret hardcoded** w `src/auth/jwt.rs:25` — `const JWT_SECRET` zamiast z `Config`
2. **Brak `decode_token()`** — nie da się walidować tokenów
3. **Brak middleware Auth** — endpointy nie są chronione
4. **Role hardcoded** na `Passenger` w `handlers/auth.rs:49,77,131`
5. **Brak zapisu GPS do bazy** — WebSocket broadcastuje ale nie persystuje
6. **Brak handlerów CRUD** dla routes, stops, vehicles, students
7. **Brak warstwy błędów** (error module) — każdy handler sam buduje `ErrorResponse`
8. **`AppState` podzielony** — `pool` i `ws_state` jako oddzielne States (problem z Axum)

---

## Faza 0 — Dokończenie infrastruktury <a id="faza-0"></a>

**Status**: 90% ukończona
**Estymacja**: 1-2 dni
**Priorytet**: KRYTYCZNY (blokuje wszystko)

### Zadania

#### 0.1 — Deploy w Coolify (ręczny krok)
- **Typ**: Operacja manualna (użytkownik)
- **Instrukcja**: Patrz `status.md` sekcja "Instrukcja deploymentu w Coolify UI"

#### 0.2 — Zunifikowany `AppState`
- **Pliki do zmiany**: `src/main.rs`, `src/handlers/auth.rs`
- **Plik do utworzenia**: `src/state.rs`
- **Opis**: Stworzyć struct `AppState` łączący `PgPool`, `Arc<WsState>`, `Arc<Config>`, `Option<redis::Client>`
- **Implementacja**:
  ```rust
  // src/state.rs
  use sqlx::PgPool;
  use std::sync::Arc;
  use crate::config::Config;
  use crate::websocket::state::WsState;

  #[derive(Clone)]
  pub struct AppState {
      pub db: PgPool,
      pub ws: Arc<WsState>,
      pub config: Arc<Config>,
  }
  ```
- **Wpływ**: Wszystkie handlery przechodzą na `State<AppState>` zamiast `State<PgPool>`

#### 0.3 — Moduł błędów
- **Plik do utworzenia**: `src/errors.rs`
- **Opis**: Centralny typ błędu `AppError` implementujący `IntoResponse`
- **Implementacja**:
  ```rust
  // src/errors.rs
  use axum::{http::StatusCode, response::IntoResponse, Json};
  use serde_json::json;

  pub enum AppError {
      BadRequest(String),
      Unauthorized(String),
      Forbidden(String),
      NotFound(String),
      Conflict(String),
      Internal(String),
      Database(sqlx::Error),
      Validation(validator::ValidationErrors),
  }

  impl IntoResponse for AppError {
      fn into_response(self) -> axum::response::Response {
          let (status, message) = match self {
              Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
              Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
              Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
              Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
              Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
              Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
              Self::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
              Self::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
          };
          (status, Json(json!({"error": message}))).into_response()
      }
  }
  ```
- **Wpływ**: Handlery zwracają `Result<Json<T>, AppError>` zamiast `Result<Json<T>, (StatusCode, Json<ErrorResponse>)>`

#### 0.4 — Aktualizacja `lib.rs`
- **Plik**: `src/lib.rs`
- **Dodać**: `pub mod state;` i `pub mod errors;`

### Weryfikacja Fazy 0

```bash
# Kompilacja
cargo build --release
# Testy (gdy będą)
cargo test
# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Faza 1 — Auth v2 <a id="faza-1"></a>

**Estymacja**: 1 tydzień
**Zależności**: Faza 0 (AppState, errors)
**Priorytet**: KRYTYCZNY

### Zadania

#### 1.1 — Naprawa JWT (wyciągnięcie secretu z Config)
- **Plik**: `src/auth/jwt.rs`
- **Zmiany**:
  - Usunąć `const JWT_SECRET` i `const JWT_EXPIRATION`
  - `generate_token_pair()` przyjmuje `&Config` jako argument
  - Dodać `pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError>`
  - Dodać `pub fn refresh_access_token(refresh_token: &str, secret: &str) -> Result<TokenPair, AppError>`

#### 1.2 — Middleware autentykacji
- **Plik do utworzenia**: `src/auth/middleware.rs`
- **Opis**: Axum extractor `AuthUser` implementujący `FromRequestParts`
- **Implementacja**:
  ```rust
  // src/auth/middleware.rs
  use axum::{extract::FromRequestParts, http::request::Parts};
  use crate::{auth::jwt::{Claims, decode_token}, errors::AppError, state::AppState};

  pub struct AuthUser(pub Claims);

  #[axum::async_trait]
  impl FromRequestParts<AppState> for AuthUser {
      type Rejection = AppError;
      async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
          let header = parts.headers.get("Authorization")
              .and_then(|v| v.to_str().ok())
              .ok_or(AppError::Unauthorized("Missing Authorization header".into()))?;
          let token = header.strip_prefix("Bearer ")
              .ok_or(AppError::Unauthorized("Invalid token format".into()))?;
          let claims = decode_token(token, &state.config.jwt_secret)?;
          Ok(AuthUser(claims))
      }
  }
  ```

#### 1.3 — Middleware ról
- **Plik do utworzenia**: `src/auth/middleware.rs` (rozszerzenie)
- **Opis**: Guard `RequireRole` — wrapper sprawdzający rolę usera
- **Implementacja**: Struct `RequireRole<const ROLES: &'static [UserRole]>` lub middleware `layer`

#### 1.4 — MFA (TOTP)
- **Plik do utworzenia**: `src/auth/mfa.rs`
- **Nowa zależność w Cargo.toml**: `totp-rs = "5"` (lub `google-authenticator`)
- **Endpointy**:
  - `POST /auth/mfa/setup` → generuje TOTP secret + QR code URI
  - `POST /auth/mfa/verify` → weryfikuje kod i aktywuje MFA
  - `POST /auth/mfa/validate` → walidacja kodu przy logowaniu
- **Flow**: Login → jeśli `mfa_enabled=true` → zwróć `mfa_required: true` → klient wysyła kod → walidacja → token

#### 1.5 — Naprawa handlera register/login
- **Plik**: `src/handlers/auth.rs`
- **Zmiany**:
  - Przejście na `State<AppState>`
  - Pobieranie roli z requesta (opcjonalnie) zamiast hardcoded `Passenger`
  - Pobieranie `role` z DB w loginie (SELECT `role` z tabeli `users`)
  - Obsługa MFA flow
  - Dodać endpoint `POST /auth/refresh` (odświeżanie tokenu)
  - Dodać endpoint `POST /auth/logout` (opcjonalnie: blacklist tokenu w Redis)

#### 1.6 — Szyfrowanie PII (AES-256)
- **Plik do utworzenia**: `src/auth/encryption.rs`
- **Nowa zależność**: `aes-gcm = "0.10"` lub `ring`
- **Opis**: Encrypt/decrypt PII (email, phone, first_name, last_name) at rest
- **Config**: dodać `ENCRYPTION_KEY` do `Config`

#### 1.7 — Migracja SQL: tabela refresh tokens
- **Plik**: `migrations/0003_auth_tokens.sql`
- **Zawartość**:
  ```sql
  CREATE TABLE refresh_tokens (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
      token_hash VARCHAR(255) NOT NULL,
      expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
      revoked BOOLEAN NOT NULL DEFAULT false,
      created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
  CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
  ```

### Weryfikacja Fazy 1

```bash
# Unit testy auth
cargo test --lib auth::
# Integration test: register → login → access protected endpoint
cargo test --test auth_integration
# Sprawdzenie kompilacji
cargo clippy -- -D warnings
```
- **Test manualny**: curl/httpie do endpointów auth (register, login, refresh, MFA setup, MFA validate)

---

## Faza 2 — System biletowy QR <a id="faza-2"></a>

**Estymacja**: 2 tygodnie
**Zależności**: Faza 1 (Auth middleware)
**Priorytet**: WYSOKI

### Zadania

#### 2.1 — Migracja SQL: bilety
- **Plik**: `migrations/0004_tickets.sql`
- **Tabele**:
  ```sql
  CREATE TYPE ticket_type AS ENUM ('single', 'daily', 'weekly', 'monthly', 'student_monthly');
  CREATE TYPE ticket_status AS ENUM ('active', 'used', 'expired', 'revoked');

  CREATE TABLE tickets (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID NOT NULL REFERENCES users(id),
      ticket_type ticket_type NOT NULL,
      status ticket_status NOT NULL DEFAULT 'active',
      route_id UUID REFERENCES routes(id),  -- NULL = sieciowy
      valid_from TIMESTAMP WITH TIME ZONE NOT NULL,
      valid_until TIMESTAMP WITH TIME ZONE NOT NULL,
      qr_token VARCHAR(512) NOT NULL UNIQUE,
      qr_token_version INT NOT NULL DEFAULT 1,
      qr_last_refreshed_at TIMESTAMP WITH TIME ZONE,
      hmac_signature VARCHAR(128) NOT NULL,
      payment_id UUID,  -- FK do payments (Faza 3)
      price_cents INT NOT NULL,
      created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_tickets_user ON tickets(user_id);
  CREATE INDEX idx_tickets_qr ON tickets(qr_token);
  CREATE INDEX idx_tickets_status ON tickets(status);
  ```

#### 2.2 — Model Rust
- **Plik do utworzenia**: `src/models/ticket.rs`
- **Structs**: `Ticket`, `TicketType`, `TicketStatus`, `CreateTicketRequest`, `TicketResponse`, `QrPayload`

#### 2.3 — Generator QR
- **Plik do utworzenia**: `src/services/qr.rs`
- **Nowe zależności**: `qrcode = "0.14"`, `image = "0.25"`, `hmac = "0.12"`, `sha2 = "0.10"`
- **Funkcjonalność**:
  - `generate_qr_payload(ticket: &Ticket) -> QrPayload` — JWT z: `ticket_id`, `type`, `valid_until`, `version`
  - `sign_qr(payload: &str, key: &[u8]) -> String` — HMAC-SHA256
  - `generate_qr_image(payload: &str) -> Vec<u8>` — PNG
  - `refresh_qr(ticket_id: Uuid) -> QrPayload` — nowy token, wersja++

#### 2.4 — Weryfikacja QR (offline-capable)
- **Plik do utworzenia**: `src/services/qr_verifier.rs`
- **Funkcjonalność**:
  - `verify_qr(token: &str, hmac_key: &[u8]) -> Result<QrPayload, AppError>` — dekoduj JWT + sprawdź HMAC + sprawdź ważność
  - Offline: weryfikacja oparta wyłącznie na kryptografii (bez zapytania do DB)

#### 2.5 — Handlery biletowe
- **Plik do utworzenia**: `src/handlers/tickets.rs`
- **Endpointy**:
  - `POST /api/v1/tickets` — kup bilet (wymaga Auth)
  - `GET /api/v1/tickets` — lista biletów użytkownika
  - `GET /api/v1/tickets/:id` — szczegóły biletu
  - `GET /api/v1/tickets/:id/qr` — pobierz aktualny QR (PNG)
  - `POST /api/v1/tickets/:id/refresh-qr` — odśwież QR token
  - `POST /api/v1/tickets/verify` — weryfikacja QR (kierowca/kontroler)
  - `POST /api/v1/tickets/:id/use` — oznacz jako użyty (przy skanowaniu)

#### 2.6 — Lifecycle management
- **Plik do utworzenia**: `src/services/ticket_lifecycle.rs`
- **Opis**: Background task (tokio::spawn) sprawdzający co minutę wygasłe bilety i oznaczający je jako `expired`

### Nowe moduły do dodania

- `src/services/mod.rs` — nowy folder serwisów
- `src/services/qr.rs`
- `src/services/qr_verifier.rs`
- `src/services/ticket_lifecycle.rs`

### Weryfikacja Fazy 2

```bash
cargo test --lib services::qr
cargo test --lib models::ticket
cargo test --test ticket_integration
```
- **Test manualny**: Kup bilet → pobierz QR → zeskanuj (verify endpoint) → sprawdź cykl życia

---

## Faza 3 — Płatności Stripe <a id="faza-3"></a>

**Estymacja**: 1 tydzień
**Zależności**: Faza 2 (bilety)
**Priorytet**: WYSOKI

### Zadania

#### 3.1 — Config Stripe
- **Plik**: `src/config.rs`
- **Dodać pola**: `stripe_secret_key`, `stripe_publishable_key`, `stripe_webhook_secret`
- **Env vars**: `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY`, `STRIPE_WEBHOOK_SECRET`

#### 3.2 — Migracja SQL: płatności
- **Plik**: `migrations/0005_payments.sql`
- **Tabele**:
  ```sql
  CREATE TYPE payment_status AS ENUM ('pending', 'succeeded', 'failed', 'refunded');

  CREATE TABLE payments (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID NOT NULL REFERENCES users(id),
      ticket_id UUID REFERENCES tickets(id),
      stripe_payment_intent_id VARCHAR(255) UNIQUE,
      stripe_subscription_id VARCHAR(255),
      amount_cents INT NOT NULL,
      currency VARCHAR(3) NOT NULL DEFAULT 'PLN',
      status payment_status NOT NULL DEFAULT 'pending',
      created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_payments_user ON payments(user_id);
  CREATE INDEX idx_payments_stripe ON payments(stripe_payment_intent_id);
  ```

#### 3.3 — Model Rust
- **Plik do utworzenia**: `src/models/payment.rs`

#### 3.4 — Serwis Stripe
- **Plik do utworzenia**: `src/services/stripe.rs`
- **Nowa zależność**: `async-stripe = { version = "0.37", features = ["runtime-tokio-hyper"] }`
- **Funkcjonalność**:
  - `create_payment_intent(amount: i64, currency: &str, metadata: HashMap) -> PaymentIntent`
  - `create_subscription(customer_id: &str, price_id: &str) -> Subscription`
  - `handle_webhook(payload: &[u8], sig: &str) -> WebhookEvent`

#### 3.5 — Handlery płatności
- **Plik do utworzenia**: `src/handlers/payments.rs`
- **Endpointy**:
  - `POST /api/v1/payments/create-intent` — tworzy PaymentIntent
  - `POST /api/v1/payments/webhook` — obsługa webhooków Stripe (bez auth!)
  - `GET /api/v1/payments/history` — historia płatności użytkownika
  - `POST /api/v1/payments/subscribe` — subskrypcja biletu miesięcznego

#### 3.6 — Retry logic
- **Opis**: W przypadku niepowodzenia płatności — retry z exponential backoff (3 próby)

### Weryfikacja Fazy 3

```bash
cargo test --lib services::stripe
# Test z Stripe CLI (mock webhooków)
stripe listen --forward-to localhost:3000/api/v1/payments/webhook
stripe trigger payment_intent.succeeded
```

---

## Faza 4 — GPS v2 <a id="faza-4"></a>

**Estymacja**: 1 tydzień
**Zależności**: Faza 0 (AppState)
**Priorytet**: WYSOKI

### Zadania

#### 4.1 — Zapis GPS do bazy
- **Plik**: `src/websocket/handler.rs`
- **Zmiana**: W `gps_update` — dodać zapis do tabeli `gps_positions` przez `AppState.db`
- **Wymaganie**: WebSocket handler musi mieć dostęp do `PgPool`

#### 4.2 — Filtr Kalmana
- **Plik do utworzenia**: `src/services/kalman.rs`
- **Opis**: Wygładzanie pozycji GPS — 1D filtr Kalmana na latitude/longitude oddzielnie
- **Parametry**: `process_noise = 0.01`, `measurement_noise = 5.0` (tunable)

#### 4.3 — R-tree indexing
- **Plik do utworzenia**: `src/services/spatial.rs`
- **Nowa zależność**: `rstar = "0.12"`
- **Opis**: In-memory R-tree przystanków dla szybkiego wyszukiwania najbliższego przystanku (geofence matching)

#### 4.4 — Autentykacja WebSocket
- **Plik**: `src/websocket/mod.rs`
- **Zmiana**: Wymagać JWT token jako query parameter `?token=xxx` przy połączeniu WS
- **Implementacja**: Walidacja tokenu przed upgrade do WebSocket

#### 4.5 — REST endpointy GPS
- **Plik do utworzenia**: `src/handlers/gps.rs`
- **Endpointy**:
  - `GET /api/v1/vehicles/:id/position` — ostatnia pozycja pojazdu
  - `GET /api/v1/vehicles/:id/history?from=&to=` — historia GPS
  - `GET /api/v1/routes/:id/vehicles` — wszystkie pojazdy na trasie (live)

### Weryfikacja Fazy 4

```bash
cargo test --lib services::kalman
cargo test --lib services::spatial
# WS test: wscat -c ws://localhost:3000/ws?token=JWT_TOKEN
```

---

## Faza 5 — Moduł szkolny <a id="faza-5"></a>

**Estymacja**: 2 tygodnie
**Zależności**: Faza 1 (Auth), Faza 4 (GPS v2)
**Priorytet**: ŚREDNI

### Zadania

#### 5.1 — Migracja SQL: moduł szkolny
- **Plik**: `migrations/0006_school_module.sql`
- **Tabele**:
  ```sql
  CREATE TABLE schools (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      name VARCHAR(255) NOT NULL,
      address TEXT,
      location GEOGRAPHY(POINT, 4326),
      contact_email VARCHAR(255),
      is_active BOOLEAN NOT NULL DEFAULT true,
      created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
  );

  CREATE TABLE boarding_events (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      student_id UUID NOT NULL REFERENCES students(id),
      vehicle_id UUID NOT NULL REFERENCES vehicles(id),
      assignment_id UUID REFERENCES vehicle_assignments(id),
      stop_id UUID REFERENCES stops(id),
      event_type VARCHAR(10) NOT NULL, -- 'board' lub 'alight'
      recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
      verified_by UUID REFERENCES users(id), -- opiekun
      is_at_correct_stop BOOLEAN,
      alert_sent BOOLEAN NOT NULL DEFAULT false
  );
  CREATE INDEX idx_boarding_student ON boarding_events(student_id, recorded_at DESC);
  ```

#### 5.2 — CRUD uczniów
- **Plik do utworzenia**: `src/handlers/students.rs`
- **Endpointy**:
  - `POST /api/v1/students` — dodaj ucznia (Admin/Parent)
  - `GET /api/v1/students` — lista (Admin)
  - `GET /api/v1/students/:id` — szczegóły
  - `PUT /api/v1/students/:id` — edycja
  - `POST /api/v1/students/:id/link-parent` — powiąż z rodzicem (invite token)
  - `GET /api/v1/parents/:id/students` — uczniowie rodzica

#### 5.3 — Geofencing
- **Plik do utworzenia**: `src/services/geofence.rs`
- **Opis**: Użyj PostGIS `ST_DWithin` do sprawdzenia czy pojazd jest w promieniu przystanku
- **Trigger**: Gdy pojazd wjedzie w geofence przystanku → sprawdź uczniów którzy mają tam wysiadać

#### 5.4 — Logika wsiadania/wysiadania
- **Plik do utworzenia**: `src/services/boarding.rs`
- **Flow**:
  1. Opiekun skanuje kartę/QR ucznia przy wsiadaniu → `boarding_event(board)`
  2. System sprawdza `default_stop_id` ucznia
  3. Gdy pojazd zbliża się do przystanku ucznia → powiadomienie
  4. Jeśli uczeń próbuje wysiąść na złym przystanku → alert do rodzica i opiekuna
  5. Opiekun potwierdza wysiadkę → `boarding_event(alight)`

#### 5.5 — Powiadomienia push
- **Plik do utworzenia**: `src/services/notifications.rs`
- **Nowa zależność**: `firebase-rs` lub `fcm` (Firebase Cloud Messaging)
- **Typy powiadomień**:
  - Uczeń wsiadł do autobusu
  - Uczeń wysiadł na przystanku
  - Alert: próba wysiadki na złym przystanku
  - Autobus zbliża się do przystanku ucznia (ETA < 5 min)

### Weryfikacja Fazy 5

```bash
cargo test --lib services::geofence
cargo test --lib services::boarding
cargo test --test school_integration
```

---

## Faza 6 — ETA i predykcja <a id="faza-6"></a>

**Estymacja**: 1 tydzień
**Zależności**: Faza 4 (GPS v2)
**Priorytet**: ŚREDNI

### Zadania

#### 6.1 — Algorytm Haversine
- **Plik do utworzenia**: `src/services/geo.rs`
- **Funkcje**:
  - `haversine_distance(lat1, lon1, lat2, lon2) -> f64` — dystans w metrach
  - `bearing(lat1, lon1, lat2, lon2) -> f64` — kierunek w stopniach

#### 6.2 — Segmentacja trasy
- **Plik do utworzenia**: `src/services/route_segments.rs`
- **Opis**: Podział trasy na segmenty (stop-to-stop), obliczenie odległości dla każdego

#### 6.3 — Predykcja ETA
- **Plik do utworzenia**: `src/services/eta.rs`
- **Algorytm**:
  1. Znajdź najbliższy segment na trasie (bieżąca pozycja GPS)
  2. Oblicz dystans do kolejnego przystanku
  3. Użyj średniej prędkości z ostatnich N pozycji GPS
  4. ETA = dystans / średnia_prędkość
  5. Opcjonalnie: koryguj na podstawie historycznych danych (ta sama pora dnia, dzień tygodnia)

#### 6.4 — Endpoint ETA
- **Plik**: `src/handlers/gps.rs` (rozszerzenie)
- **Endpointy**:
  - `GET /api/v1/vehicles/:id/eta` — ETA do każdego przystanku na trasie
  - `GET /api/v1/stops/:id/arrivals` — oczekiwane przyjazdy na przystanek

### Weryfikacja Fazy 6

```bash
cargo test --lib services::geo
cargo test --lib services::eta
```

---

## Faza 7 — Admin API <a id="faza-7"></a>

**Estymacja**: 1 tydzień
**Zależności**: Faza 1 (Auth + role guard)
**Priorytet**: ŚREDNI

### Zadania

#### 7.1 — CRUD Trasy
- **Plik do utworzenia**: `src/handlers/routes.rs`
- **Endpointy** (wymaga `RequireRole::Admin`):
  - `POST /api/v1/admin/routes` — utwórz trasę z przystankami
  - `GET /api/v1/admin/routes` — lista tras
  - `PUT /api/v1/admin/routes/:id` — edycja
  - `DELETE /api/v1/admin/routes/:id` — deaktywacja (soft delete)

#### 7.2 — CRUD Przystanki
- **Plik do utworzenia**: `src/handlers/stops.rs`
- **Endpointy** (Admin):
  - `POST /api/v1/admin/stops`
  - `GET /api/v1/admin/stops`
  - `PUT /api/v1/admin/stops/:id`
  - `DELETE /api/v1/admin/stops/:id`
  - `GET /api/v1/stops/nearby?lat=&lon=&radius=` — publiczny (PostGIS `ST_DWithin`)

#### 7.3 — CRUD Pojazdy
- **Plik do utworzenia**: `src/handlers/vehicles.rs`
- **Endpointy** (Admin):
  - `POST /api/v1/admin/vehicles`
  - `GET /api/v1/admin/vehicles`
  - `PUT /api/v1/admin/vehicles/:id`
  - `POST /api/v1/admin/vehicles/:id/assign` — przypisz do trasy
  - `DELETE /api/v1/admin/vehicles/:id/assign` — odpisz

#### 7.4 — Zarządzanie użytkownikami
- **Plik do utworzenia**: `src/handlers/admin_users.rs`
- **Endpointy** (Admin):
  - `GET /api/v1/admin/users` — lista z filtrowaniem po roli
  - `PUT /api/v1/admin/users/:id/role` — zmiana roli
  - `PUT /api/v1/admin/users/:id/status` — aktywacja/deaktywacja

#### 7.5 — Dashboard & raporty
- **Plik do utworzenia**: `src/handlers/admin_dashboard.rs`
- **Endpointy**:
  - `GET /api/v1/admin/dashboard` — statystyki: aktywne pojazdy, pasażerowie online, przychody
  - `GET /api/v1/admin/reports/revenue?from=&to=` — raport przychodów
  - `GET /api/v1/admin/reports/usage?from=&to=` — usage statistics

#### 7.6 — Audit trail
- **Migracja**: `migrations/0007_audit_trail.sql`
  ```sql
  CREATE TABLE audit_logs (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID REFERENCES users(id),
      action VARCHAR(50) NOT NULL,
      entity_type VARCHAR(50) NOT NULL,
      entity_id UUID,
      old_value JSONB,
      new_value JSONB,
      ip_address VARCHAR(45),
      created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_audit_user ON audit_logs(user_id);
  CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
  ```
- **Middleware/service**: Automatyczne logowanie zmian w tabelach administracyjnych

### Weryfikacja Fazy 7

```bash
cargo test --test admin_integration
```
- Test manualny: Zaloguj jako Admin → CRUD na routes, stops, vehicles → sprawdź audit log

---

## Faza 8 — Mobile v2 <a id="faza-8"></a>

**Estymacja**: 2 tygodnie
**Zależności**: Fazy 1-6 (backend gotowy)
**Priorytet**: ŚREDNI
**Technologia**: React Native (Expo) — istniejący setup w `mobile/`

> **Uwaga**: Ten plan dotyczy backendu (endpointy REST/WS). Zmiany w `mobile/` to oddzielny plan frontendowy.

### Niezbędne endpointy backendowe dla Mobile

- Faza 1: Auth endpoints (register, login, MFA)
- Faza 2: Ticket endpoints (kup, pokaż QR, odśwież)
- Faza 3: Payment endpoints (Stripe)
- Faza 4: GPS/WS endpoints (tracking na żywo)
- Faza 5: School endpoints (boarding, powiadomienia)
- Faza 6: ETA endpoints

### API versioning
- **Prefix**: `/api/v1/` dla wszystkich endpointów
- **Router nesting** w `main.rs`:
  ```rust
  let api_v1 = Router::new()
      .nest("/auth", auth_routes())
      .nest("/tickets", ticket_routes())
      .nest("/payments", payment_routes())
      .nest("/vehicles", vehicle_routes())
      .nest("/routes", route_routes())
      .nest("/stops", stop_routes())
      .nest("/students", student_routes())
      .nest("/admin", admin_routes());
  let app = Router::new()
      .nest("/api/v1", api_v1)
      .route("/ws", get(ws_handler))
      .route("/health", get(health_check));
  ```

---

## Faza 9 — Tauri Admin Panel <a id="faza-9"></a>

**Estymacja**: 2 tygodnie
**Zależności**: Faza 7 (Admin API)
**Priorytet**: NISKI

> **Uwaga**: Desktop app w Tauri + React. Ten plan dotyczy wyłącznie backendu Rust.
> Panel admina konsumuje Admin API z Fazy 7. Nie wymaga dodatkowych zmian w backendzie.

---

## Faza 10 — RODO / Compliance <a id="faza-10"></a>

**Estymacja**: 1 tydzień
**Zależności**: Faza 1 (encryption), Faza 7 (audit)
**Priorytet**: KONIECZNY PRZED PRODUKCJĄ

### Zadania

#### 10.1 — Szyfrowanie danych w spoczynku
- Upewnić się, że `encryption.rs` (Faza 1.6) szyfruje: `email`, `phone`, `first_name`, `last_name` w tabeli `users` oraz dane uczniów

#### 10.2 — Zarządzanie zgodami
- **Migracja**: `migrations/0008_consents.sql`
  ```sql
  CREATE TABLE user_consents (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      user_id UUID NOT NULL REFERENCES users(id),
      consent_type VARCHAR(50) NOT NULL, -- 'data_processing', 'marketing', 'location_tracking'
      granted BOOLEAN NOT NULL,
      granted_at TIMESTAMP WITH TIME ZONE,
      revoked_at TIMESTAMP WITH TIME ZONE,
      ip_address VARCHAR(45),
      created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
  );
  ```
- **Endpointy**:
  - `GET /api/v1/user/consents` — lista zgód
  - `POST /api/v1/user/consents` — udziel/odwołaj zgodę

#### 10.3 — Prawo do usunięcia danych (Art. 17 RODO)
- **Endpoint**: `DELETE /api/v1/user/account` — anonimizacja danych (nie usuwaj rekordu, zastąp PII placeholderami)

#### 10.4 — Eksport danych (Art. 20 RODO)
- **Endpoint**: `GET /api/v1/user/data-export` — JSON z wszystkimi danymi użytkownika

#### 10.5 — Minimalizacja danych
- Background job: Usuwanie pozycji GPS starszych niż 90 dni
- Retencja audit logów: 1 rok

### Weryfikacja Fazy 10

```bash
cargo test --test rodo_integration
```
- Test manualny: Eksport danych → usunięcie konta → sprawdź anonimizację w DB

---

## Faza 11 — Testy i optymalizacja <a id="faza-11"></a>

**Estymacja**: 1 tydzień
**Zależności**: Wszystkie poprzednie fazy
**Priorytet**: KONIECZNY PRZED PRODUKCJĄ

### Zadania

#### 11.1 — Unit testy
- Katalog: `tests/unit/`
- Pokrycie: auth (JWT, password, MFA), QR (generate, verify), Kalman, Haversine, ETA
- **Komenda**: `cargo test --lib`

#### 11.2 — Integration testy
- Katalog: `tests/integration/`
- Wymagane: testowa baza PostgreSQL (docker-compose.test.yml)
- Scenariusze:
  - Auth flow: register → login → MFA → refresh → protected endpoint
  - Ticket flow: kup bilet → QR → weryfikacja → wygaszenie
  - GPS flow: WS connect → send GPS → verify DB storage → verify broadcast
  - School flow: dodaj ucznia → boarding → alert → powiadomienie
- **Komenda**: `cargo test --test '*'`

#### 11.3 — Load testy
- **Narzędzie**: `drill` lub `k6` lub `criterion` (benchmark)
- Scenariusze: 100 równoczesnych WS connections wysyłających GPS co 15s

#### 11.4 — Security audit
- Sprawdź: rate limiting (tower-governor), CORS, Content-Security-Policy, SQL injection (sqlx = safe), XSS (API only)
- **Nowa zależność**: `tower-governor = "0.4"` (rate limiting)

#### 11.5 — Dokumentacja API
- **Nowa zależność**: `utoipa = "4"`, `utoipa-swagger-ui = "7"`
- Dodaj `#[utoipa::path]` do wszystkich handlerów
- Endpoint: `GET /swagger-ui/` — interaktywna dokumentacja

### Weryfikacja Fazy 11

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo audit  # security audit dependencies
```

---

## Konwencje i reguły kodu <a id="konwencje"></a>

### Struktura plików

```
src/
├── main.rs              # Router setup, server startup
├── lib.rs               # Re-exporty modułów
├── state.rs             # AppState (PgPool + WsState + Config)
├── config.rs            # Konfiguracja z env
├── errors.rs            # Centralny typ AppError
├── auth/
│   ├── mod.rs
│   ├── jwt.rs           # Token generation/validation
│   ├── password.rs      # Bcrypt hash/verify
│   ├── middleware.rs     # AuthUser extractor, RequireRole
│   ├── mfa.rs           # TOTP setup/verify
│   └── encryption.rs    # AES-256 PII encryption
├── models/
│   ├── mod.rs           # BaseEntity + re-export
│   ├── user.rs
│   ├── student.rs
│   ├── stop.rs
│   ├── route.rs
│   ├── vehicle.rs
│   ├── gps.rs
│   ├── ticket.rs
│   └── payment.rs
├── handlers/
│   ├── mod.rs           # Re-export + route builders
│   ├── auth.rs
│   ├── tickets.rs
│   ├── payments.rs
│   ├── gps.rs
│   ├── routes.rs
│   ├── stops.rs
│   ├── vehicles.rs
│   ├── students.rs
│   ├── admin_users.rs
│   └── admin_dashboard.rs
├── services/
│   ├── mod.rs
│   ├── qr.rs
│   ├── qr_verifier.rs
│   ├── ticket_lifecycle.rs
│   ├── stripe.rs
│   ├── kalman.rs
│   ├── spatial.rs
│   ├── geo.rs
│   ├── route_segments.rs
│   ├── eta.rs
│   ├── geofence.rs
│   ├── boarding.rs
│   └── notifications.rs
└── websocket/
    ├── mod.rs
    ├── handler.rs
    └── state.rs
```

### Reguły kodowania

1. **Handlery** nie zawierają logiki biznesowej — delegują do `services/`
2. **Typy błędów**: Zawsze `Result<T, AppError>` — nigdy `(StatusCode, Json<..>)`
3. **SQL queries**: Zawsze typed queries z `sqlx::query_as!()` lub `sqlx::query!()` — SQLX_OFFLINE mode
4. **Nazewnictwo endpointów**: REST `kebab-case`, prefixed `/api/v1/`
5. **Logi**: `tracing::info!`, `tracing::error!` — structured logging
6. **Commits**: Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
7. **Migracje**: Numerowane sekwencyjnie `0003_`, `0004_`, … — sqlx migrate
8. **Testy**: Per-moduł unit testy + integration testy w `tests/`
9. **Po każdej fazie**: `cargo clippy -- -D warnings && cargo fmt --check && cargo test`

---

## Mapowanie zależności między fazami <a id="zaleznosci"></a>

```
Faza 0 (Infrastruktura) ──→ WSZYSTKO
        │
        ├── Faza 1 (Auth v2) ──→ Faza 2 (QR) ──→ Faza 3 (Stripe)
        │        │                                       │
        │        ├── Faza 7 (Admin API) ──→ Faza 9 (Tauri)
        │        │
        │        └── Faza 10 (RODO)
        │
        └── Faza 4 (GPS v2) ──→ Faza 6 (ETA)
                 │
                 └── Faza 5 (Moduł szkolny)

Faza 8 (Mobile) ← wymaga: Fazy 1-6 (backend ready)
Faza 11 (Testy) ← wymaga: Wszystkie fazy
```

### Sugerowana kolejność implementacji

1. **Faza 0** (1-2 dni) — AppState, errors, deploy
2. **Faza 1** (1 tyg) — Auth v2 (blokuje wszystko)
3. **Faza 4** (1 tyg) — GPS v2 (równolegle z 2 jeśli inny dev)
4. **Faza 2** (2 tyg) — Bilety QR
5. **Faza 3** (1 tyg) — Stripe
6. **Faza 7** (1 tyg) — Admin API
7. **Faza 5** (2 tyg) — Moduł szkolny
8. **Faza 6** (1 tyg) — ETA
9. **Faza 10** (1 tyg) — RODO
10. **Faza 11** (1 tyg) — Testy end-to-end
11. **Faza 8** (2 tyg) — Mobile v2
12. **Faza 9** (2 tyg) — Tauri Admin
