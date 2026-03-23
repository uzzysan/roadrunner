# RoadRunner - Status Projektu

> Data utworzenia: 2026-03-23  
> Aktualizacja: 2026-03-23 21:05  
> Aktywna faza: Faza 0 - Infrastruktura  
> Zarządzanie deploymentem: Coolify (VPS)

---

## 📊 Ogólny Progress

```
[███████░░░░░░░░░░░░░] 35% - Faza 0 w trakcie
```

| Faza | Status | Postęp | Estymowany czas |
|------|--------|--------|-----------------|
| 0. Infrastruktura | 🔄 W trakcie | 70% | 1 tydzień |
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
- [x] Docker Compose (PostgreSQL + PostGIS + Redis)

### Infrastruktura / DevOps ✅
- [x] **jcodemunch-mcp** zainstalowany i skonfigurowany (80 symboli zaindeksowanych)
- [x] **Coolify MCP** skonfigurowany (projekt utworzony: vxad36z1njjiwcvn909ow8en)
- [x] **SQLx CLI** zainstalowane (v0.8.6)
- [x] Migracje przekonwertowane do formatu sqlx (0001_, 0002_)
- [x] Struktura testów utworzona (tests/unit, tests/integration)
- [x] **GitHub Actions CI** workflow (.github/workflows/ci.yml)
- [x] **Dockerfile** dla produkcji (multi-stage build)
- [x] **docker-compose.prod.yml** dla deploymentu
- [x] Cargo.toml zaktualizowany (lib, bin, sqlx migrate)

### Mobile
- [x] Expo + React Native setup
- [x] Bottom navigation
- [x] Dark/Light theme
- [x] Brand colors (pomarańczowy/szary)

---

## 🔄 Aktualna Faza (0): Infrastruktura - Pozostałe Zadania

#### SQLx Setup (Finalizacja)
- [ ] Przygotowanie `sqlx-data.json` (cargo sqlx prepare)
- [ ] Weryfikacja wszystkich zapytań SQL w czasie kompilacji
- [ ] Testy z sqlx offline mode

#### Testy
- [ ] Uruchomienie testów jednostkowych (cargo test)
- [ ] Naprawa ewentualnych błędów kompilacji
- [ ] Dodanie więcej testów auth

#### Deployment w Coolify
- [ ] Skonfigurowanie aplikacji w Coolify UI (git repo)
- [ ] Dodanie PostgreSQL + PostGIS jako usługa
- [ ] Dodanie Redis jako usługa
- [ ] Konfiguracja environment variables
- [ ] Pierwszy deployment

---

## 📋 Szczegółowy Plan Fazy 0

### Tydzień 1: Infrastruktura - STATUS

| Dzień | Zadanie | Status |
|-------|---------|--------|
| 1 | Setup jcodemunch-mcp | ✅ |
| 1 | Konfiguracja Coolify MCP | ✅ |
| 1 | Utworzenie projektu w Coolify | ✅ |
| 2 | SQLx CLI instalacja | ✅ |
| 2 | Migracje do sqlx format | ✅ |
| 2 | Struktura testów | ✅ |
| 2 | GitHub Actions workflow | ✅ |
| 2 | Dockerfile + docker-compose.prod | ✅ |
| 3 | Finalizacja SQLx (prepare) | 🔄 |
| 3 | Testy kompilacji | ⏳ |
| 4 | Coolify deployment konfiguracja | ⏳ |
| 5 | Dokumentacja deploymentu | ⏳ |

---

## 📝 Notatki i Decyzje

### Architektura
- Backend: Rust (Axum) + SQLx + PostgreSQL/PostGIS
- Frontend Mobile: React Native (Expo)
- Frontend Admin: Tauri + React
- Deployment: Coolify (VPS na https://coolify.maculewicz.pro)

### Konwencje kodu
- Używamy MCP `jcodemunch` do analizy kodu (80 symboli zaindeksowanych)
- Status projektu aktualizowany w `status.md`
- Commits: konwencja conventional commits
- SQLx: compile-time checked queries

### MCP Serwery
- **jcodemunch-mcp**: Skonfigurowany w `~/.config/kimi/mcp.json`
  - Zaindeksowano: 80 symboli
  - Lokalizacja: `/home/uzzy/.code-index/local-roadrunner-d4befb17`
- **coolify-mcp**: Skonfigurowany w `~/.config/kimi/mcp.json`
  - URL: https://coolify.maculewicz.pro
  - Projekt: RoadRunner (vxad36z1njjiwcvn909ow8en)
  - Środowisko: production (hhsvf6it2kpyf60pqwq1magf)

### Deployment
- **Dockerfile**: Multi-stage build z Rust + sqlx-cli
- **docker-compose.prod.yml**: App + PostgreSQL + Redis
- **GitHub Actions**: CI z testami, fmt, clippy, sqlx prepare check

---

## 🔗 Linki

- Repo: https://github.com/uzzysan/roadrunner
- Coolify: https://coolify.maculewicz.pro
- Coolify Project: vxad36z1njjiwcvn909ow8en
- Staging: (do skonfigurowania)
- Prod: (do skonfigurowania)
