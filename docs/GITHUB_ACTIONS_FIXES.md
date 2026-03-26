# RoadRunner - GitHub Actions Issues & Fixes

**Data**: 2026-03-26  
**Autor**: HAL9000  
**Status**: 🔧 W trakcie naprawy

---

## Znalezione Problemy

### 1. ❌ CI Workflow - Zły obraz PostgreSQL **(NAPRAWIONE)**
**Plik**: `.github/workflows/ci.yml`

**Problem**: Workflow używał `postgres:16` zamiast `postgis/postgis:16-3.4`

**Skutek**: Migracje używające typów PostGIS (`GEOGRAPHY(POINT, 4326)`) failowały

**Fix**: Zmieniono obraz na `postgis/postgis:16-3.4`

**Commit**: `c585590` - fix(ci): use postgis image instead of plain postgres

---

### 2. ❌ SQLX Offline Files - Niepoprawne/Niekompletne **(DO NAPRAWY)**
**Plik**: `.sqlx/query-*.json`

**Problem**: 
- Pliki `.sqlx` są niepoprawne - jeden ma nazwę `query-*.json` (z gwiazdką!)
- Struktura JSON jest uszkodzona
- Brakuje wielu zapytań używanych w kodzie

**Skutek**: 
- `cargo build` z `SQLX_OFFLINE=true` failuje
- Workflow `prepare-sqlx.yml` nie może wygenerować poprawnych plików
- Wszystkie workflow failują

**Rozwiązanie wymagane**:
1. Usunąć błędne pliki `.sqlx/query-*.json`
2. Uruchomić lokalnie: `cargo sqlx prepare` z działającą bazą
3. Zacommitować wygenerowane pliki
4. Sprawdzić czy `cargo build` działa z `SQLX_OFFLINE=true`

**Tymczasowe obejście**: 
- Można ustawić `SQLX_OFFLINE=false` w CI i polegać na bazie danych
- ALE to spowolni buildy i wymaga działającej bazy

---

### 3. ❌ Docker Build - Zależność od SQLX_OFFLINE **(ZALEŻNE OD #2)**
**Plik**: `Dockerfile`

**Problem**: Dockerfile ustawia `SQLX_OFFLINE=true` ale pliki `.sqlx` są niepoprawne

**Skutek**: Docker build failuje przy kompilacji

**Fix**: Naprawić problem #2

---

## Podsumowanie Statusu Actions

| Workflow | Status | Przyczyna |
|----------|--------|-----------|
| CI | ❌ Fail | Problem #2 - SQLX files |
| Docker Build | ❌ Fail | Problem #2 - SQLX files |
| Prepare SQLX | ❌ Fail | Problem #2 - nie może wygenerować plików |

**Po naprawie #1 (PostGIS)**: CI przejdzie migracje, ale zatrzyma się na buildzie  
**Wymagana naprawa #2**: Regeneracja plików `.sqlx`

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
