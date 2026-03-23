# RoadRunner - Status Projektu

> Data utworzenia: 2026-03-23  
> Aktualizacja: 2026-03-23 21:30  
> Aktywna faza: Faza 0 - Infrastruktura  
> Zarządzanie deploymentem: Coolify (VPS)

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
- [x] Docker Compose (PostgreSQL + PostGIS + Redis)

### Infrastruktura / DevOps ✅
- [x] **jcodemunch-mcp** zainstalowany i skonfigurowany (80 symboli zaindeksowanych)
- [x] **Coolify MCP** skonfigurowany (projekt utworzony: vxad36z1njjiwcvn909ow8en)
- [x] **SQLx CLI** zainstalowane (v0.8.6)
- [x] Migracje przekonwertowane do formatu sqlx (0001_, 0002_)
- [x] **SQLX_OFFLINE** skonfigurowane - build przechodzi bez bazy danych
- [x] Struktura testów utworzona (tests/unit, tests/integration)
- [x] **GitHub Actions CI** workflow (.github/workflows/ci.yml)
- [x] **Dockerfile** dla produkcji (multi-stage build, SQLX_OFFLINE)
- [x] **docker-compose.prod.yml** dla deploymentu
- [x] **coolify.json** - konfiguracja deploymentu
- [x] Cargo.toml zaktualizowany (lib, bin, sqlx migrate)
- [x] Build testowany: `cargo build --release` ✅

### Mobile
- [x] Expo + React Native setup
- [x] Bottom navigation
- [x] Dark/Light theme
- [x] Brand colors (pomarańczowy/szary)

---

## 🔄 Aktualna Faza (0): Infrastruktura - Deployment w Coolify

### Instrukcja deploymentu w Coolify UI:

#### 1. Dodanie PostgreSQL + PostGIS:
1. Wejdź w projekt **RoadRunner** w Coolify
2. Kliknij **+ New** → **Database**
3. Wybierz **PostgreSQL**
4. Ustaw:
   - Name: `roadrunner-db`
   - Version: `16` (lub latest)
5. W sekcji **PostGIS** (jeśli dostępna) włącz rozszerzenie
6. Zapisz i uruchom

#### 2. Dodanie Redis:
1. **+ New** → **Database**
2. Wybierz **Redis**
3. Name: `roadrunner-redis`
4. Zapisz i uruchom

#### 3. Dodanie aplikacji RoadRunner:
1. **+ New** → **Application**
2. Wybierz **Git Repository**
3. Repo: `https://github.com/uzzysan/roadrunner`
4. Branch: `main`
5. Build Pack: `Docker Compose`
6. Docker Compose File: `docker-compose.prod.yml`
7. Environment Variables:
   ```
   DB_USER=roadrunner
   DB_PASSWORD=[generate-strong-password]
   DB_NAME=roadrunner
   DATABASE_URL=postgres://roadrunner:[password]@roadrunner-db:5432/roadrunner
   REDIS_URL=redis://roadrunner-redis:6379
   JWT_SECRET=[generate-64-char-secret]
   JWT_EXPIRATION=86400
   PORT=3000
   RUST_LOG=info
   ```
8. Zapisz i deploy

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
| 3 | SQLX_OFFLINE setup | ✅ |
| 3 | Build testowany | ✅ |
| 4 | Coolify deployment (ręczny) | 🔄 |
| 5 | Dokumentacja deploymentu | ✅ |

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
- SQLx: compile-time checked queries (offline mode dla buildów)

### MCP Serwery
- **jcodemunch-mcp**: Skonfigurowany w `~/.config/kimi/mcp.json`
  - Zaindeksowano: 80 symboli
  - Lokalizacja: `/home/uzzy/.code-index/local-roadrunner-d4befb17`
- **coolify-mcp**: Skonfigurowany w `~/.config/kimi/mcp.json`
  - URL: https://coolify.maculewicz.pro
  - Projekt: RoadRunner (vxad36z1njjiwcvn909ow8en)
  - Środowisko: production (hhsvf6it2kpyf60pqwq1magf)

### Deployment
- **Dockerfile**: Multi-stage build z SQLX_OFFLINE=true
- **docker-compose.prod.yml**: App + PostgreSQL + Redis
- **GitHub Actions**: CI z testami, fmt, clippy
- **Coolify**: Projekt utworzony, czeka na deployment aplikacji

### Ważne zmiany
- Dodano `src/lib.rs` dla umożliwienia testów
- Zaktualizowano `Cargo.toml` z sekcjami [lib] i [[bin]]
- Dodano `.sqlx/query-*.json` dla offline builds
- Dockerfile używa `SQLX_OFFLINE=true`

---

## 🔗 Linki

- Repo: https://github.com/uzzysan/roadrunner
- Coolify: https://coolify.maculewicz.pro
- Coolify Project: vxad36z1njjiwcvn909ow8en
- Staging: (do skonfigurowania w Coolify)
- Prod: (do skonfigurowania w Coolify)
