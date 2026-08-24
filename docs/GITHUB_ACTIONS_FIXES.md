# RoadRunner - GitHub Actions Issues & Fixes

**Data**: 2026-03-26 (zaktualizowano 2026-08-24)  
**Autor**: HAL9000  
**Status**: ✅ Problem #2 i #3 naprawione 2026-08-24 (patrz `docs/status-log.md` w repo `RoadRunner` za pełny kontekst tej sesji)

---

## Znalezione Problemy

### 1. ❌ CI Workflow - Zły obraz PostgreSQL **(NAPRAWIONE)**
**Plik**: `.github/workflows/ci.yml`

**Problem**: Workflow używał `postgres:16` zamiast `postgis/postgis:16-3.4`

**Skutek**: Migracje używające typów PostGIS (`GEOGRAPHY(POINT, 4326)`) failowały

**Fix**: Zmieniono obraz na `postgis/postgis:16-3.4`

**Commit**: `c585590` - fix(ci): use postgis image instead of plain postgres

---

### 2. ✅ SQLX Offline Files - Niepoprawne/Niekompletne **(NAPRAWIONE 2026-08-24)**
**Plik**: `.sqlx/query-*.json`

**Problem**: 
- Pliki `.sqlx` były niepoprawne - jeden miał dosłowną nazwę `query-*.json` (z gwiazdką w nazwie
  pliku, nie jako wzorzec!), co dodatkowo psuło indeksowanie repo na Windows (nielegalny znak w
  nazwie pliku)
- Brakowało większości zapytań używanych w kodzie (tylko 2 pliki w cache)

**Skutek**: 
- `cargo build` z `SQLX_OFFLINE=true` failował
- Workflow `prepare-sqlx.yml` nie mógł wygenerować poprawnych plików

**Fix**: usunięto błędny plik `query-*.json`, postawiono lokalnie Postgres 16 + PostGIS,
zaaplikowano wszystkie 9 migracji, uruchomiono `cargo sqlx prepare` z działającym
`DATABASE_URL` — wygenerowano kompletny, poprawny cache (20 plików). Zweryfikowano:
`SQLX_OFFLINE=true cargo check --all-targets` przechodzi czysto.

---

### 3. ✅ Docker Build - Zależność od SQLX_OFFLINE **(NAPRAWIONE — zależało od #2)**
**Plik**: `Dockerfile`

**Problem**: Dockerfile ustawia `SQLX_OFFLINE=true` ale pliki `.sqlx` były niepoprawne

**Skutek**: Docker build failował przy kompilacji

**Fix**: naprawione wraz z problemem #2 — cache `.sqlx` jest teraz kompletny i poprawny.

---

## Podsumowanie Statusu Actions

| Workflow | Status (2026-08-24) | Przyczyna |
|----------|--------|-----------|
| CI | ✅ Powinien przejść | Problemy #1, #2, #3 naprawione; nie uruchomiono realnie w Actions w tej sesji (brak dostępu do CI z tego środowiska) — zweryfikowano lokalnie ekwiwalentem (`cargo check --all-targets`, `SQLX_OFFLINE=true cargo check`, `cargo test --lib`) |
| Docker Build | ✅ Powinien przejść | j.w. |
| Prepare SQLX | ✅ Niepotrzebny do odblokowania builda | cache `.sqlx` już kompletny; workflow nadal wart utrzymania jako auto-regeneracja przy zmianach zapytań |

**Zweryfikowano lokalnie 2026-08-24**: `cargo check --all-targets` czysto (0 błędów), 35/35
testów jednostkowych przechodzi, `SQLX_OFFLINE=true cargo check` czysto.

---

## Rekomendacje

### Krótkoterminowe (teraz):
1. ✅ Naprawić obraz PostgreSQL w CI **(ZROBIONE)**
2. 🔧 Wygenerować poprawne pliki `.sqlx/query-*.json`
3. 🔧 Przetestować build lokalnie

### Średnioterminowe:
1. Dodać `cargo sqlx prepare --check` do CI żeby wykrywać niezsynchronizowane pliki
2. Rozważyć użycie `sqlx migrate run` w Dockerfile zamiast w entrypoint
3. Dodać cache dla `target/` w GitHub Actions

---

## Komendy do naprawy SQLX

```bash
# 1. Start bazy lokalnie
docker-compose up -d postgres

# 2. Ustawić DATABASE_URL
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/roadrunner_dev"

# 3. Uruchomić migracje
cargo sqlx database create
cargo sqlx migrate run

# 4. Wygenerować pliki .sqlx
cargo sqlx prepare

# 5. Sprawdzić czy build działa
SQLX_OFFLINE=true cargo build --release

# 6. Zacommitować
git add .sqlx/
git commit -m "fix(sqlx): regenerate offline query files"
git push
```

---

**Następny krok**: Wygenerować poprawne pliki `.sqlx/query-*.json` i spushować
