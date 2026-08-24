# RoadRunner - Status Projektu

> Data utworzenia: 2026-03-23  
> Aktualizacja: 2026-03-23 21:30 (deployment sekcja usunięta 2026-08-24 — patrz niżej)  
> Aktywna faza: Faza 0 - Infrastruktura  
> Zarządzanie deploymentem: GitHub Actions → OVH VPS (Podman) — patrz `docs/status-log.md` w repo `RoadRunner`
>
> **2026-08-24:** Coolify porzucone na rzecz GitHub Actions + Podman na OVH VPS (decyzja
> architektoniczna, patrz `RoadRunner/docs/architecture.md` §9). `coolify.json` i
> `docker-compose.prod.yml` usunięte z repo; sekcja instrukcji deploymentu w Coolify usunięta
> poniżej — została tylko jako wpis w historii commitów.

---

## 📊 Ogólny Progress

```
[█████████░░░░░░░░░░░] 45% - Faza 0 w trakcie
```

| Faza | Status | Postęp | Estymowany czas |
|------|--------|--------|-----------------|
| 0. Infrastruktura | 🔄 W trakcie | 90% | 1 tydzień |
| 1. Auth v2 | ⏳ Oczekuje | 0% | 1 tydzień |
| 2. Bilety QR | ⏳ Oczekuje | 0% | 2 tygodnie |
| 3. Stripe | ⏳ Oczekuje | 0% | 1 tydzień |
| 4. GPS v2 | ⏳ Oczekuje | 0% | 1 tydzień |
| 5. Moduł szkolny | ⏳ Oczekuje | 0% | 2 tygodnie |
| 6. ETA | ⏳ Oczekuje | 0% | 1 tydzień |
| 7. Admin API | ⏳ Oczekuje | 0% | 1 tydzień |
| 8. Mobile v2 | ⏳ Oczekuje | 0% | 2 tygodnie |
| 9. Tauri Admin | ⏳ Oczekuje | 0% | 2 tygodnie |
| 10. RODO | ⏳ Oczekuje | 0% | 1 tydzień |
| 11. Testy | ⏳ Oczekuje | 0% | 1 tydzień |

---

## ✅ Zaimplementowane (Faza 0)

### Backend
- [x] Struktura projektu Rust (Axum)
- [x] Modele danych (User, Student, Stop, Route, Vehicle, GPS)
- [x] Podstawowa autentykacja JWT
- [x] WebSocket handler (podstawowy)
- [x] Migracje SQL (users, transport schema)
- [x] Docker Compose (PostgreSQL + PostGIS; Redis dropped 2026-08-24, unused)

### Infrastruktura / DevOps ✅
- [x] **jcodemunch-mcp** zainstalowany i skonfigurowany (80 symboli zaindeksowanych)
- [x] **SQLx CLI** zainstalowane (v0.8.6)
- [x] Migracje przekonwertowane do formatu sqlx (0001_, 0002_)
- [x] **SQLX_OFFLINE** skonfigurowane - build przechodzi bez bazy danych (cache zregenerowany 2026-08-24)
- [x] Struktura testów utworzona (tests/unit, tests/integration)
- [x] **GitHub Actions CI** workflow (.github/workflows/ci.yml)
- [x] **Dockerfile** dla produkcji (multi-stage build, SQLX_OFFLINE)
- [x] Cargo.toml zaktualizowany (lib, bin, sqlx migrate)
- [x] Build testowany: `cargo build --release` ✅

### Mobile
- [x] Expo + React Native setup
- [x] Bottom navigation
- [x] Dark/Light theme
- [x] Brand colors (pomarańczowy/szary)

---

## 🔄 Aktualna Faza (0): Infrastruktura

Deployment plan (Coolify UI walkthrough that used to live here) removed 2026-08-24 — superseded
by GitHub Actions → OVH VPS via Podman. See `RoadRunner/docs/architecture.md` §9 and
`RoadRunner/docs/development-plan.md` Phase 7 for the current plan.

---

## 📋 Szczegółowy Plan Fazy 0

### Tydzień 1: Infrastruktura - STATUS

| Dzień | Zadanie | Status |
|-------|---------|--------|
| 1 | Setup jcodemunch-mcp | ✅ |
| 2 | SQLx CLI instalacja | ✅ |
| 2 | Migracje do sqlx format | ✅ |
| 2 | Struktura testów | ✅ |
| 2 | GitHub Actions workflow | ✅ |
| 2 | Dockerfile | ✅ |
| 3 | SQLX_OFFLINE setup | ✅ |
| 3 | Build testowany | ✅ |
| 4 | GitHub Actions deployment do OVH VPS (Podman) | 🔄 patrz RoadRunner/docs |
| 5 | Dokumentacja deploymentu | ✅ |

---

## 📝 Notatki i Decyzje

### Architektura
- Backend: Rust (Axum) + SQLx + PostgreSQL/PostGIS
- Frontend Mobile: Flutter + flutter_rust_bridge (decyzja 2026-08-24 — patrz RoadRunner/docs/status-log.md; ten repo ma prototyp React Native, zachowany tylko jako referencja UX)
- Frontend Admin: Leptos web dashboard (decyzja 2026-08-24, zamiast Tauri)
- Deployment: GitHub Actions → OVH VPS (Podman), decyzja 2026-08-24

### Konwencje kodu
- Używamy MCP `jcodemunch` do analizy kodu (80 symboli zaindeksowanych)
- Status projektu aktualizowany w `status.md`
- Commits: konwencja conventional commits
- SQLx: compile-time checked queries (offline mode dla buildów)

### MCP Serwery
- **jcodemunch-mcp**: Skonfigurowany w `~/.config/kimi/mcp.json`
  - Zaindeksowano: 80 symboli
  - Lokalizacja: `/home/uzzy/.code-index/local-roadrunner-d4befb17`

### Deployment
- **Dockerfile**: Multi-stage build z SQLX_OFFLINE=true
- **GitHub Actions**: CI z testami, fmt, clippy; deployment workflow do OVH VPS w budowie
  (patrz `RoadRunner/docs/development-plan.md` Faza 7)

### Ważne zmiany
- Dodano `src/lib.rs` dla umożliwienia testów
- Zaktualizowano `Cargo.toml` z sekcjami [lib] i [[bin]]
- Dodano `.sqlx/query-*.json` dla offline builds
- Dockerfile używa `SQLX_OFFLINE=true`
- 2026-08-24: usunięto `coolify.json`, `docker-compose.prod.yml` i Redis (nieużywany w kodzie)

---

## 🔗 Linki

- Repo: https://github.com/uzzysan/roadrunner
