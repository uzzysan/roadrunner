# RoadRunner - Analiza Projektu i Plan Pracy

## 📊 Podsumowanie Stanu Projektu

### Zaimplementowane ✅

#### Backend (Rust/Axum)
| Komponent | Status | Opis |
|-----------|--------|------|
| Struktura projektu | ✅ | Modularna architektura (models, handlers, auth, websocket) |
| Konfiguracja | ✅ | Config z env variables (dotenvy) |
| Baza danych | ✅ | PostgreSQL + PostGIS, 2 migracje SQL |
| Modele danych | ✅ | User, Student, Stop, Route, Vehicle, GpsPosition + relacje |
| Autentykacja JWT | ✅ | Access + refresh tokens, bcrypt, middleware |
| WebSocket | ✅ | Podstawowa obsługa, typy klientów, broadcast GPS |
| Health check | ✅ | Endpoint /health |

#### Mobile (React Native/Expo)
| Komponent | Status | Opis |
|-----------|--------|------|
| Struktura Expo | ✅ | Podstawowa konfiguracja |
| Nawigacja | ✅ | Bottom tabs (Home, Tickets, Tracking, Profile) |
| Theme | ✅ | Dark/light mode, brand colors (pomarańczowy/szary) |
| Ekran Home | 🟡 | Mock data, bez integracji API |
| Pozostałe ekrany | 🟡 | Placeholdery |

### W Trakcie/Draft 🟡
- WebSocket handler (podstawowy, bez filtra Kalmana)
- Modele danych (zdefiniowane, bez pełnej integracji)

### Do Zaimplementowania ❌

#### Krytyczne (MVP)
1. **MFA dla rodziców** - TOTP/QR code setup
2. **Szyfrowanie PII** - AES-256 dla danych wrażliwych
3. **System biletowy QR** - Generowanie, HMAC, weryfikacja offline
4. **Płatności Stripe** - Payment Intents, webhooks, subskrypcje
5. **Geofencing** - Walidacja przystanków, alerty
6. **Logika szkolna** - Wsiadanie/wysiadanie, weryfikacja celu

#### Ważne (Post-MVP)
7. **ETA i predykcja** - Algorytm Haversine, historia
8. **Panel admina** - Tauri + API zarządzania
9. **Push notifications** - Firebase/APNs
10. **Filtr Kalmana** - Wygładzanie GPS

#### Infrastruktura
11. **SQLx compile-time checks** - Weryfikacja zapytań w czasie kompilacji
12. **Testy** - Unit, integration, load tests
13. **CI/CD** - GitHub Actions, deployment
14. **Dokumentacja API** - OpenAPI/Swagger

---

## 📋 Szczegółowy Plan Pracy

### Faza 0: Infrastruktura (1 tydzień)
**Cel**: Stabilna baza pod dalszy rozwój

- [ ] Skonfigurować SQLx compile-time checks
- [ ] Dodać migracje SQLx (`sqlx migrate`)
- [ ] Utworzyć strukturę testów
- [ ] Setup CI/CD (GitHub Actions)
- [ ] Poprawić obsługę błędów (thiserror/anyhow)
- [ ] Dodać request logging (tracing)

### Faza 1: Autentykacja v2 (1 tydzień)
**Cel**: Pełna obsługa użytkowników z MFA

- [ ] Implementacja MFA (TOTP)
  - Endpoint setup MFA (generowanie QR)
  - Weryfikacja kodu TOTP przy logowaniu
  - Recovery codes
- [ ] Szyfrowanie PII (email, phone)
  - Implementacja AES-256-GCM
  - Wrappery dla pól wrażliwych
- [ ] Role-based access control (RBAC)
  - Middleware dla ról (admin, driver, parent)
  - Ograniczenia endpointów
- [ ] Refresh token rotation

### Faza 2: System Biletowy QR (2 tygodnie)
**Cel**: Funkcjonalny system biletów

- [ ] Model danych biletów
  - Ticket, TicketType, TicketValidation
- [ ] Generowanie QR
  - Biblioteka `qrcode` lub `qirust`
  - JWT w QR (id, typ, ważność, HMAC)
- [ ] HMAC podpis
  - Klucz serwera do podpisu
  - Weryfikacja offline (klucz publiczny w aplikacji kierowcy)
- [ ] Dynamiczne odświeżanie
  - Timer w aplikacji mobilnej
  - Nowy QR co 30s
- [ ] API biletów
  - Zakup (przygotowanie pod Stripe)
  - Lista aktywnych
  - Historia
- [ ] Weryfikacja (aplikacja kierowcy)
  - Skanowanie QR
  - Walidacja HMAC + czasu

### Faza 3: Płatności Stripe (1 tydzień)
**Cel**: Pełna integracja płatności

- [ ] Konfiguracja async-stripe
- [ ] Payment Intents
  - Tworzenie płatności przy zakupie
  - Potwierdzenie z frontendu
- [ ] Webhook handler
  - `payment_intent.succeeded`
  - `payment_intent.payment_failed`
- [ ] Subskrypcje (bilety miesięczne)
  - Stripe Subscription
  - Automatyczne odnowienia
- [ ] Retry logic
  - Dead letter queue dla failed webhooks

### Faza 4: GPS i WebSocket v2 (1 tydzień)
**Cel**: Stabilny tracking z optymalizacją

- [ ] Zapisywanie pozycji do bazy
  - Batch insert co 15s
  - Retencja danych (30 dni)
- [ ] Filtr Kalmana (opcjonalnie)
  - Wygładzanie trajektorii
- [ ] R-tree indexing
  - Indeks przestrzenny dla zapytań
- [ ] Heartbeat/keepalive
  - Timeout dla nieaktywnych kierowców
- [ ] API pozycji
  - GET /vehicles/nearby?lat=&lng=&radius=
  - GET /vehicles/:id/location

### Faza 5: Moduł Szkolny (2 tygodnie)
**Cel**: Pełna obsługa transportu szkolnego

- [ ] Zarządzanie uczniami
  - CRUD studentów (admin)
  - Powiązanie z rodzicami
  - Default stop
- [ ] Geofencing
  - Weryfikacja czy GPS w promieniu przystanku
  - Alert przy wysiadce poza strefą
- [ ] Logika wsiadania/wysiadania
  - Student scans QR przy wejściu
  - System śledzi aktualny przystanek
- [ ] Weryfikacja przystanku docelowego
  - Porównanie z default_stop_id
  - Alert jeśli inny przystanek
- [ ] Powiadomienia push (przygotowanie)
  - Schemat powiadomień
  - Integracja z Firebase

### Faza 6: ETA i Predykcja (1 tydzień)
**Cel**: Dokładne szacowanie przyjazdu

- [ ] Algorytm Haversine
  - Odległość do następnego przystanku
- [ ] Obliczanie ETA
  - Na podstawie prędkości i odległości
  - Aktualizacja co 15s
- [ ] Historia przejazdów
  - Zapisywanie czasów między przystankami
  - Średnie czasy dla tras
- [ ] Predykcja oparta na historii
  - Uwzględnienie godziny/dnia
  - Korekta ETA na podstawie historii

### Faza 7: Panel Administratora API (1 tydzień)
**Cel**: Zarządzanie systemem

- [ ] Zarządzanie flotą
  - CRUD pojazdów
  - Przypisanie kierowców
- [ ] Zarządzanie trasami
  - CRUD tras i przystanków
  - Kolejność przystanków
- [ ] Monitorowanie GPS
  - Widok wszystkich pojazdów
  - Historia tras
- [ ] Raporty
  - Sprzedaż biletów
  - Statystyki przejazdów
  - Audit log

### Faza 8: Mobile App v2 (2 tygodnie)
**Cel**: Funkcjonalna aplikacja pasażera

- [ ] Autentykacja
  - Ekrany login/register
  - Integracja z API
  - Przechowywanie tokenów
- [ ] Bilety QR
  - Wyświetlanie QR
  - Odliczanie do odświeżenia
  - Historia biletów
- [ ] Mapa (śledzenie)
  - Integracja z mapą (Mapbox/Google)
  - Pokazywanie pojazdów na żywo
  - WebSocket client
- [ ] Zakup biletów
  - Lista typów biletów
  - Flow płatności (Stripe SDK)
- [ ] Profil
  - Dane użytkownika
  - Powiązani uczniowie (dla rodziców)

### Faza 9: Panel Admin (Tauri) (2 tygodnie)
**Cel**: Desktop app dla administratorów

- [ ] Setup Tauri + React
- [ ] Dashboard
  - Statystyki w czasie rzeczywistym
  - Mapa z pojazdami
- [ ] Zarządzanie użytkownikami
- [ ] Zarządzanie trasami (z mapą)
- [ ] Raporty i wykresy

### Faza 10: Bezpieczeństwo i RODO (1 tydzień)
**Cel**: Zgodność i bezpieczeństwo

- [ ] Szyfrowanie danych w spoczynku (DB)
- [ ] Zarządzanie zgodami RODO
- [ ] Anonimizacja danych historycznych
- [ ] Dokumentacja RODO

### Faza 11: Testy i Optymalizacja (1 tydzień)
**Cel**: Stabilna produkcja

- [ ] Unit testy (backend >70%)
- [ ] Integration testy (API)
- [ ] Load testy (WebSocket, GPS)
- [ ] Security audit
- [ ] Dokumentacja API (OpenAPI)
- [ ] Performance optimization

---

## 📅 Proponowany Timeline

| Faza | Czas | Sprint |
|------|------|--------|
| 0. Infrastruktura | 1 tydzień | 1 |
| 1. Auth v2 | 1 tydzień | 2 |
| 2. Bilety QR | 2 tygodnie | 3-4 |
| 3. Stripe | 1 tydzień | 5 |
| 4. GPS v2 | 1 tydzień | 6 |
| 5. Moduł szkolny | 2 tygodnie | 7-8 |
| 6. ETA | 1 tydzień | 9 |
| 7. Admin API | 1 tydzień | 10 |
| 8. Mobile v2 | 2 tygodnie | 11-12 |
| 9. Tauri Admin | 2 tygodnie | 13-14 |
| 10. RODO | 1 tydzień | 15 |
| 11. Testy | 1 tydzień | 16 |

**Całkowity czas**: ~16 tygodni (4 miesiące)

---

## 🎯 Priorytety (MVP w 8 tygodni)

Aby mieć działający MVP w 8 tygodni, skupić się na:

1. **Week 1-2**: Infrastruktura + Auth + QR (bez MFA)
2. **Week 3-4**: Stripe + Bilety (zakup + weryfikacja)
3. **Week 5-6**: GPS v2 + Podstawowy moduł szkolny
4. **Week 7-8**: Mobile v2 (Auth + Bilety + Podstawowy tracking)

**MVP Scope**:
- Rejestracja/logowanie użytkowników
- Zakup biletów (Stripe)
- Wyświetlanie QR w aplikacji
- Skanowanie QR przez kierowcę
- Podstawowy GPS tracking (bez predykcji)
- Podstawowy moduł szkolny (bez geofencingu)
