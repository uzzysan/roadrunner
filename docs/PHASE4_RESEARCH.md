# Faza 4: Zarządzanie Flotą - Research & Wymagania

> **Agent:** Dev_Research (Fleet Management Specialist)  
> **Cel:** Dopracowanie funkcjonalności zarządzania flotą dla systemu transportowego

---

## 🎯 Zakres Researchu

### 1. Podstawowa Funkcjonalność Floty
- [ ] Zarządzanie pojazdami (dodawanie, edycja, usuwanie)
- [ ] Zarządzanie kierowcami (profile, uprawnienia, grafiki)
- [ ] Przypisywanie kierowców do pojazdów
- [ ] Śledzenie statusu pojazdów (aktywny, w serwisie, awaria)

### 2. GPS Tracking & Real-time
- [ ] Lokalizacja GPS pojazdów w czasie rzeczywistym
- [ ] Wyświetlanie pozycji autobusów na mapie dla użytkowników
- [ ] Obliczanie ETA (szacowany czas przyjazdu)
- [ ] Historia tras pojazdów

### 3. System Awarii & Zastępstw
- [ ] Zgłaszanie awarii przez kierowcę (mobile app)
- [ ] Automatyczne powiadomienia o opóźnieniach dla pasażerów
- [ ] Panel do przypisywania pojazdów zastępczych
- [ ] Śledzenie statusu naprawy

### 4. Powiadomienia dla Rodziców (School Transport)
- [ ] Rejestracja dziecka w systemie
- [ ] Przypisanie dziecka do kierowcy/trasy
- [ ] Powiadomienie o wsiadaniu (QR scan lub manualne potwierdzenie)
- [ ] Powiadomienie o wysiadaniu z godziną i przystankiem
- [ ] Historia przejazdów dziecka

### 5. Panel Administracyjny
- [ ] Dashboard z overview floty
- [ ] Zarządzanie incydentami
- [ ] Raporty i statystyki
- [ ] Zarządzanie powiadomieniami

---

## 📋 Szczegółowe Wymagania do Zdefiniowania

### 3.1 Zgłaszanie Awarii
```
Pytania do rozwiązania:
- Jakie pola w formularzu awarii? (typ, opis, priorytet, zdjęcia?)
- Czy kierowca może zgłosić "opóźnienie" bez awarii?
- Jakie są kategorie awarii? (mechaniczna, trasa, inne)
- Czy awaria automatycznie blokuje pojazd w systemie?
```

### 3.2 Pojazdy Zastępcze
```
Pytania do rozwiązania:
- Skąd system wie, które pojazdy są dostępne jako zastępcze?
- Czy zastępczy pojazd musi być tego samego typu/pojemności?
- Jak przepisać pasażerów/kursy na nowy pojazd?
- Czy zastępstwo wpływa na rozkład jazdy?
```

### 4.1 Powiadomienia dla Rodziców
```
Pytania do rozwiązania:
- Jak rodzic potwierdza, że to jego dziecko? (QR, NFC, PIN?)
- Czy kierowca skanuje kod dziecka czy manualnie klika?
- Jakie kanały powiadomień? (push, SMS, email)
- Czy rodzic widzi lokalizację autobusu w czasie rzeczywistym?
- Co jeśli dziecko nie wsiadło (nieobecność)?
```

---

## 🔗 Integracje do Rozważenia

### GPS Tracking
- [ ] GPS w pojazdach (integracja z urządzeniami GPS)
- [ ] WebSocket/SSE dla real-time updates
- [ ] Geofencing (alerty przy wjeździe/wyjeździe z strefy)

### Powiadomienia
- [ ] Push notifications (Firebase Cloud Messaging)
- [ ] SMS gateway (opcjonalnie)
- [ ] Email notifications

### Mapy
- [ ] OpenStreetMap dla wyświetlania pozycji pojazdów
- [ ] Routing i ETA calculation

---

## 📊 Propozycja Modeli Danych

### Vehicle (Pojazd)
```rust
struct Vehicle {
    id: UUID,
    registration_number: String,  // Rejestracja
    vin: String,                  // Numer VIN
    brand: String,                // Marka
    model: String,                // Model
    year: i32,                    // Rok produkcji
    capacity: i32,                // Pojemność pasażerska
    type: VehicleType,            // Autobus, minibus, etc.
    status: VehicleStatus,        // Active, Maintenance, Broken
    gps_device_id: Option<String>,// ID urządzenia GPS
    current_location: Option<Point>, // Aktualna lokalizacja
    current_driver_id: Option<UUID>, // Aktualny kierowca
    created_at: DateTime,
    updated_at: DateTime,
}
```

### Driver (Kierowca)
```rust
struct Driver {
    id: UUID,
    user_id: UUID,                // Powiązanie z tabelą users
    license_number: String,       // Numer prawa jazdy
    license_categories: Vec<String>, // Kategorie
    phone: String,
    emergency_contact: String,
    status: DriverStatus,         // Active, OnLeave, Suspended
    assigned_vehicle_id: Option<UUID>,
    current_route_id: Option<UUID>,
    created_at: DateTime,
}
```

### Incident (Zdarzenie/Awaria)
```rust
struct Incident {
    id: UUID,
    vehicle_id: UUID,
    driver_id: UUID,
    incident_type: IncidentType,  // Breakdown, Delay, Accident, Other
    severity: Severity,           // Low, Medium, High, Critical
    description: String,
    location: Option<Point>,      // Gdzie wystąpiło
    reported_at: DateTime,
    resolved_at: Option<DateTime>,
    status: IncidentStatus,       // Reported, InProgress, Resolved
    replacement_vehicle_id: Option<UUID>,
    affected_routes: Vec<UUID>,
    notifications_sent: bool,
}
```

### ChildRegistration (Rejestracja Dziecka - School Transport)
```rust
struct ChildRegistration {
    id: UUID,
    parent_user_id: UUID,
    child_name: String,
    child_birth_date: Date,
    school_id: UUID,
    assigned_route_id: UUID,
    assigned_stop_id: UUID,
    qr_code: String,              // Unikalny kod QR
    status: RegistrationStatus,   // Active, Inactive
    photo_url: Option<String>,    // Zdjęcie dziecka (opcjonalne)
    created_at: DateTime,
}
```

### ChildAttendance (Obecność Dziecka)
```rust
struct ChildAttendance {
    id: UUID,
    child_id: UUID,
    route_id: UUID,
    vehicle_id: UUID,
    driver_id: UUID,
    boarding_stop_id: UUID,
    boarding_time: DateTime,
    alighting_stop_id: Option<UUID>,
    alighting_time: Option<DateTime>,
    status: AttendanceStatus,     // Boarded, Completed, Absent
    confirmed_by: ConfirmationType, // QR, Manual, GPS
}
```

### ParentNotification (Powiadomienie dla Rodzica)
```rust
struct ParentNotification {
    id: UUID,
    parent_user_id: UUID,
    child_id: UUID,
    notification_type: NotificationType, // Boarding, Alighting, Delay, Incident
    title: String,
    message: String,
    sent_at: DateTime,
    read_at: Option<DateTime>,
    channel: NotificationChannel, // Push, SMS, Email
}
```

---

## 🎨 UI/UX Do Zaprojektowania

### Dla Kierowcy (Mobile)
- [ ] Ekran główny z aktualną trasą
- [ ] Przycisk "Zgłoś awarię"
- [ ] Lista dzieci do odebrania (school transport)
- [ ] Skaner QR do potwierdzania wsiadania/wysiadania
- [ ] Status pojazdu (online/offline)

### Dla Rodzica (Mobile)
- [ ] Lista zarejestrowanych dzieci
- [ ] Szczegóły dziecka (trasa, przystanek)
- [ ] Mapa z lokalizacją autobusu
- [ ] Historia przejazdów
- [ ] Ustawienia powiadomień

### Dla Administratora (Web/Desktop)
- [ ] Dashboard z mapą wszystkich pojazdów
- [ ] Lista aktywnych incydentów
- [ ] Zarządzanie pojazdami zastępczymi
- [ ] Raporty (spóźnienia, awarie, obecność)
- [ ] Zarządzanie kierowcami

---

## ⚠️ Wyzwania Techniczne

1. **Real-time GPS Updates**
   - Jak często wysyłać lokalizację? (co 5-10s?)
   - Bateria w urządzeniu kierowcy
   - Obsługa offline (cache'owanie)

2. **Skalowalność**
   - Ile pojazdów jednocześnie? (100? 1000?)
   - WebSocket connections management
   - Database writes optimization

3. **Bezpieczeństwo Dzieci**
   - Weryfikacja tożsamości przy odbiorze
   - GDPR compliance (dane dzieci)
   - Logowanie wszystkich akcji

4. **Niezawodność**
   - Co jeśli GPS nie działa?
   - Co jeśli kierowca nie ma internetu?
   - Fallback dla powiadomień

---

## 📅 Proponowany Podział Prac

### Sprint 1: Podstawy Floty
- [ ] Modele: Vehicle, Driver
- [ ] CRUD dla pojazdów i kierowców
- [ ] Przypisywanie kierowców do pojazdów

### Sprint 2: GPS Tracking
- [ ] GPS endpointy
- [ ] WebSocket dla real-time updates
- [ ] Wyświetlanie pojazdów na mapie (mobile)

### Sprint 3: System Awarii
- [ ] Model Incident
- [ ] Zgłaszanie awarii przez kierowcę
- [ ] Automatyczne powiadomienia o opóźnieniach

### Sprint 4: Pojazdy Zastępcze
- [ ] Logika przypisywania zastępczych
- [ ] Panel administracyjny
- [ ] Aktualizacja rozkładów

### Sprint 5: School Transport (Rodzice)
- [ ] Rejestracja dzieci
- [ ] System QR kodów
- [ ] Powiadomienia dla rodziców

### Sprint 6: Panel Admina
- [ ] Dashboard
- [ ] Raporty
- [ ] Zarządzanie incydentami

---

## 📝 Notatki z Researchu

*(Agent Research powinien tu zapisać swoje ustalenia)*

### Ustalenia:
1. ...
2. ...
3. ...

### Zalecenia:
1. ...
2. ...
3. ...

---

**Status Researchu:** 🔄 W trakcie  
**Oczekiwany czas:** 2-3 godziny  
**Odpowiedzialny:** Dev_Research Agent
