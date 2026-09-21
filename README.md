# RoadRunner - System Transportu Zbiorowego i Szkolnego

System transportowy zbudowany w Rust (Axum + PostGIS) z modułem szkolnym, GPS trackingiem, biletami QR i płatnościami Stripe.

> **Źródło prawdy (2026-09-21):** to repozytorium,
> [`github.com/uzzysan/roadrunner`](https://github.com/uzzysan/roadrunner), jest jedynym
> kanonicznym repo RoadRunner. Wdrożony Axum/SQLx + Expo MVP jest utrzymywany wyłącznie
> w zakresie bezpieczeństwa i ciągłości działania, a architektura docelowa powstaje tutaj
> przez migrację etapową. Szczegóły: [`docs/architecture.md`](./docs/architecture.md),
> [`docs/development-plan.md`](./docs/development-plan.md) i
> [`docs/adr/0001-canonical-repository-and-migration.md`](./docs/adr/0001-canonical-repository-and-migration.md).

## Stack Technologiczny
- **Backend**: Rust + Axum
- **Baza danych**: PostgreSQL + PostGIS
- **Real-time**: WebSockets (GPS tracking)
- **Płatności**: Stripe (async-stripe)
- **Bezpieczeństwo**: JWT, MFA, AES-256

## Status Projektu
Zarządzanie zadaniami: [Linear](https://linear.app)

## Dokumentacja
- [Kontekst domeny i słownik](./CONTEXT.md)
- [Architektura bieżąca i docelowa](./docs/architecture.md)
- [Plan realizacji i zależności](./docs/development-plan.md)
- [Bieżący status](./docs/status-log.md)
- [Design language](./docs/design-language.md)
- [Dokumenty historyczne](./docs/PLAN.md), [status.md](./status.md), [status/STATUS.md](./status/STATUS.md)

## Uruchomienie
```bash
docker-compose up -d  # PostgreSQL + PostGIS
cargo run             # Dev server
```
