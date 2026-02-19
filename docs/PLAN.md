# Plan Pracy: RoadRunner

## Faza 0: Setup i Infrastruktura (AKTYWNA)
- [x] Inicjalizacja projektu Rust (Cargo)
- [x] Struktura katalogów
- [x] Konfiguracja Docker (PostgreSQL + PostGIS + Redis)
- [x] Pliki .env.example i .gitignore
- [ ] Instalacja i konfiguracja MCP (Linear, Context7, GitHub)
- [ ] Pierwszy commit i push do GitHub
- [ ] Setup PostgreSQL + PostGIS lokalnie
- [ ] Konfiguracja SQLx z compile-time checks
- [ ] Podstawowy serwer Axum (Hello World)

## Faza 1: Core Backend + Auth
- [ ] Modele użytkowników (pasażer, kierowca, opiekun, admin)
- [ ] System autentykacji JWT
- [ ] MFA dla kont rodziców/opiekunów
- [ ] Rejestracja i logowanie
- [ ] Middleware autoryzacji
- [ ] Szyfrowanie PII (AES-256)

## Faza 2: Moduł GPS i WebSockets
- [ ] WebSocket handler dla kierowców
- [ ] Strumień danych GPS (co 15 sekund)
- [ ] Tokio broadcast channels
- [ ] Heartbeat/Ping-Pong
- [ ] Filtr Kalmana (wygładzanie GPS)
- [ ] R-tree indexing dla przystanków

## Faza 3: System Biletowy QR
- [ ] Generowanie kodów QR (qirust)
- [ ] JWT w QR (unikalny ID, typ biletu, ważność)
- [ ] HMAC podpis
- [ ] Dynamiczne odświeżanie QR
- [ ] Weryfikacja offline
- [ ] Lifecycle management

## Faza 4: Płatności Stripe
- [ ] Integracja async-stripe
- [ ] Payment Intents
- [ ] Webhook handler
- [ ] Subskrypcje (bilety miesięczne)
- [ ] Retry logic

## Faza 5: Moduł Szkolny
- [ ] Model ucznia + relacje z rodzicami
- [ ] Geofencing przystanków
- [ ] Logika wsiadania/wysiadania
- [ ] Weryfikacja przystanku docelowego
- [ ] Alert przy próbie wysiadki
- [ ] Powiadomienia push

## Faza 6: ETA i Predykcja
- [ ] Algorytm Haversine
- [ ] Segmentacja trasy
- [ ] Predykcja oparta na historii
- [ ] Dashboard z ETA

## Faza 7: Panel Administratora
- [ ] API dla dashboardu
- [ ] Zarządzanie flotą
- [ ] Zarządzanie trasami
- [ ] Monitorowanie GPS
- [ ] Raporty sprzedaży
- [ ] Audit trails

## Faza 8: Bezpieczeństwo i RODO
- [ ] Szyfrowanie danych w spoczynku
- [ ] Zarządzanie zgodami
- [ ] Minimalizacja danych
- [ ] Dokumentacja RODO

## Faza 9: Testy i Optymalizacja
- [ ] Unit testy
- [ ] Integration testy
- [ ] Load testy
- [ ] Security audit
- [ ] Dokumentacja API
