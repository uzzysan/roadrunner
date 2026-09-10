# RoadRunner — Architecture

> **Canonical reference** for the system architecture.
> Last updated: 2026-09-11
> Maintainer: backend team

---

## §1 — System Overview

RoadRunner ("Sygnał") is a school-bus and public-transit platform for Polish carriers. It consists of:

- **Backend API** — Rust / Axum / sqlx 0.7 / PostgreSQL 15 + PostGIS
- **Mobile app** — React Native / Expo (driver, chaperone, guardian, customer views)
- **Admin panel** — Tauri + React (carrier_admin operations)
- **Deployment** — per-carrier OVH VPS, Docker + Caddy, GitHub Actions CI/CD (see §9)

---

## §2 — Data Model Summary

Core entities and their relationships:

```
carriers (implicit — one per VPS instance)
  └─ users           (all tenant roles; see §5)
  └─ vehicles        (bus fleet)
  └─ drivers         (linked to users.role = driver)
  └─ routes          (lines with ordered stops)
  └─ stops           (PostGIS POINT locations)
  └─ schedules       (planned timetable; route × stop × day_type)
  └─ trips           (actual executions; links vehicle/route/date)
  └─ trip_events     (actual arrivals/departures; written by geofence primitive)
  └─ tickets         (QR tickets with validation log)
  └─ payments        (Stripe payment records)
  └─ students        (school module; linked to guardian users)
  └─ incidents       (safety/operational incident log)
```

---

## §3 — Technology Choices

| Layer | Choice | Notes |
|---|---|---|
| Language | Rust (stable) | Axum 0.7, tokio, sqlx 0.7 |
| Database | PostgreSQL 15 + PostGIS | Spatial queries for stops / geofencing |
| Auth | JWT (HS256) + optional TOTP MFA | Short-lived access token + refresh token |
| Payments | Stripe | `async-stripe` crate |
| Realtime | WebSocket (axum `ws`) | GPS position push, future alert push |
| CI/CD | GitHub Actions → OVH VPS | Docker multi-stage build, Caddy reverse proxy |

---

## §4 — API Design

- REST/JSON over HTTPS
- No versioning prefix yet (`/auth`, `/routes`, `/stops`, `/schedules`, `/trips`, `/tickets`, `/payments`)
- Auth: `Authorization: Bearer <token>` on protected routes
- Errors: `{ "success": false, "error": "<message>", "status": <http_code> }`
- 500-class errors return a generic message — internal detail is never sent to clients

---

## §5 — Tenant RBAC Roles

Six roles in the `user_role` Postgres enum. Each VPS instance hosts exactly one carrier — there is no cross-carrier role.

| Role | DB value | Description |
|---|---|---|
| `Customer` | `customer` | End-user / ticket buyer |
| `Driver` | `driver` | Bus / tram operator |
| `Chaperone` | `chaperone` | School escort / attendant |
| `Guardian` | `guardian` | Parent or legal guardian |
| `CarrierAdmin` | `carrier_admin` | Carrier-level administrator |
| `Controller` | `controller` | Ticket controller / inspector (**port-required**: must exist before the controller app ships) |

> **`system_admin` is not a tenant role.** Under the per-VPS model (§9) there is no cross-tenant
> database for it to hold rights in. Infrastructure-level access is handled via SSH + server
> credentials, not an application role.

### Row-Level Security (RLS) note

Earlier design documents described Postgres RLS with a `carrier_id` column as the tenancy boundary. **That model is superseded (see §6).** Under §9, each VPS hosts a single carrier, so RLS is defence-in-depth on a single-tenant database rather than the isolation boundary. The `carrier_id` column and RLS policies may still be useful for future audit logging or as a secondary safety net, but they are not relied on for tenant isolation.

---

## §6 — [SUPERSEDED] Multi-Tenant Shared-Database Model

> ⚠️ **This section describes a design that was considered but never built.**
> It is retained as historical context only. The current deployment model is §9.
> All references to §6 in older documents and commit messages should be read as historical.

The original multi-tenant design proposed:

- A single shared PostgreSQL cluster serving all carriers
- A `carrier_id UUID` column on every tenant-scoped table
- Postgres Row-Level Security (RLS) policies enforcing `carrier_id = current_setting('app.carrier_id')`
- A `system_admin` role with cross-carrier visibility

**Why it was abandoned:**

1. Per-carrier VPS gives stronger isolation at acceptable cost for the target customer size (5–50 vehicles per carrier)
2. RLS adds complexity and a misconfiguration risk that outweighs the benefits at this scale
3. The operational model (Caddy + Docker per VPS) is simpler to reason about, back up, and debug

**Role list as originally specified (for reference):**

| Role | Notes |
|---|---|
| `customer` | End-user |
| `driver` | Bus operator |
| `chaperone` | School escort |
| `guardian` | Parent / legal guardian |
| `carrier_admin` | Carrier admin |
| `controller` | Ticket inspector (port-required — see §5) |

> `system_admin` was listed in earlier drafts of this section. It has been removed.
> No application-level `system_admin` role exists or will exist under the per-VPS model.

---

## §7 — Privacy and Data Retention

- `trip_events` cascade-delete with their parent `trip` record
- `trips.retain_until` defaults to `trip_date + 30 days`; a maintenance job runs `DELETE FROM trips WHERE retain_until < CURRENT_DATE`
- GPS position history (`vehicle_locations`) is not yet subject to explicit retention — a separate retention policy will be defined in Phase 7
- PII (email, phone, student data) is governed by GDPR / RODO; legal review required before Phase 3

---

## §8 — Security Boundaries

| Boundary | Mechanism |
|---|---|
| Network | Caddy TLS termination; backend not exposed directly |
| Auth | JWT verify on every protected route; `require_role!` macro enforces role |
| SQL injection | All queries use `sqlx::QueryBuilder` + `push_bind` or `query_as` with `$N` params |
| Error leakage | 500-class `AppError` variants return generic message; internal detail discarded |
| Input validation | Time/UUID/limit validation in handlers; `ValidationError` → 400 |

---

## §9 — Current Deployment Model (Canonical)

Each carrier runs an isolated VPS instance. This is the model that is actually built and deployed.

```
GitHub repo
    └─ GitHub Actions CI
           ├─ cargo test (SQLX_OFFLINE=true)
           ├─ docker build (multi-stage)
           └─ SSH deploy → OVH VPS
                  ├─ Caddy (TLS, reverse proxy → :3000)
                  ├─ roadrunner container (Axum API, port 3000)
                  └─ postgres container (PostgreSQL 15 + PostGIS)
```

**Key properties:**

- One VPS = one carrier = one database. No shared state between carriers.
- Migrations run at startup (`sqlx::migrate!()`).
- Secrets in `.env.production` on the VPS (never committed); `.env.production.example` is the tracked template.
- Rollback: redeploy previous image tag (manual; automated tag-pinning is issue #34).
- Backups: not yet automated (issue #22, priority-high).

**Relation to §6 RLS:** The `carrier_id` / RLS approach from §6 is not the isolation mechanism here. If RLS policies are added in future, they serve as defence-in-depth, not as the tenancy boundary.
