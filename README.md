# RoadRunner - System Transportu Zbiorowego i Szkolnego

System transportowy zbudowany w Rust (Axum + PostGIS) z modułem szkolnym, GPS trackingiem, biletami QR i płatnościami Stripe.

## Stack Technologiczny
- **Backend**: Rust + Axum
- **Baza danych**: PostgreSQL + PostGIS
- **Real-time**: WebSockets (GPS tracking)
- **Płatności**: Stripe (async-stripe)
- **Bezpieczeństwo**: JWT, MFA, AES-256

## Status Projektu
Zarządzanie zadaniami: [Linear](https://linear.app)

## Dokumentacja
- [Plan pracy](./docs/PLAN.md)
- [Architektura](./docs/ARCHITECTURE.md)

## Uruchomienie
```bash
docker-compose up -d  # PostgreSQL + PostGIS
cargo run             # Dev server
```
