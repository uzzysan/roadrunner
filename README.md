# RoadRunner - System Transportu Zbiorowego i Szkolnego

System transportowy zbudowany w Rust (Axum + PostGIS) z modułem szkolnym, GPS trackingiem, biletami QR i płatnościami Stripe.

> **2026-08-24:** ten kod jest bazą, na której budowany jest projekt dalej, ale bieżąca
> architektura i plan pracy żyją teraz w osobnym repo `RoadRunner`
> (`docs/architecture.md`, `docs/development-plan.md`, `docs/status-log.md` — tam też pełny
> zapis decyzji podjętych 2026-08-24: Flutter zamiast React Native na mobile, Leptos zamiast
> Tauri na panel admina, OVH VPS + Podman zamiast Coolify/Raspberry Pi jako cel deploymentu).
> Ten plik i pozostałe dokumenty w `docs/`/`status/` zostają jako zapis historyczny tego, co
> faktycznie zbudowano i dlaczego — nie są już aktualizowane jako plan na przyszłość.

## Stack Technologiczny
- **Backend**: Rust + Axum
- **Baza danych**: PostgreSQL + PostGIS
- **Real-time**: WebSockets (GPS tracking)
- **Płatności**: Stripe (async-stripe)
- **Bezpieczeństwo**: JWT, MFA, AES-256

## Status Projektu
Zarządzanie zadaniami: [Linear](https://linear.app)

## Dokumentacja
- [Plan pracy (historyczny)](./docs/PLAN.md)
- [Status i decyzje (2026-08-24)](./docs/GITHUB_ACTIONS_FIXES.md), [status.md](./status.md), [status/STATUS.md](./status/STATUS.md)

## Uruchomienie
```bash
docker-compose up -d  # PostgreSQL + PostGIS
cargo run             # Dev server
```
