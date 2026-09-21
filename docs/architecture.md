# RoadRunner architecture

Status: current architecture authority, updated 2026-09-21.

## Repository and delivery model

`https://github.com/uzzysan/roadrunner` is the only canonical repository. A former local
rewrite checkout had no remote and no longer exists, so it must not be referenced as a source
of truth. The target architecture is delivered by a staged, in-place migration in this repo.

The `main` branch must remain releasable. Deployed-MVP fixes and target work share CI, schema
history and release traceability, but every Linear issue declares its delivery track.
Changes are delivered from short-lived Linear issue branches through reviewed pull requests;
the repository-wide fallback owner is defined in `.github/CODEOWNERS`.

## Current deployed MVP

- One Rust 2021 crate using Axum 0.7 and SQLx 0.7.
- PostgreSQL/PostGIS 16/3.4, started locally with `docker-compose.yaml`.
- JWT authentication, MFA helpers, QR tickets, Stripe integration and WebSocket scaffolding.
- React Native/Expo application under `mobile/`.
- GitHub Actions CI and an OVH deployment path.

This is an operational baseline, not proof that target-rewrite phases are complete. Until
cutover it receives only security, privacy, correctness and availability work tracked under
the `Deployed MVP` label.

## Target architecture

The target is a Cargo workspace with explicit capability boundaries:

- `domain`: tenant-aware entities, policy and shared value objects;
- `api`: Axum HTTP composition, request context and public contracts;
- `gps-tracking`: authenticated ingest, geofencing, ETA and live subscriptions;
- `ticketing`: products, signed presentations, validation and reconciliation;
- `payments`: Stripe lifecycle and ticket activation;
- `school-safety`: consent, assignments, attendance and alerts.

The carrier is the tenant boundary. Every tenant-owned row has a non-null `carrier_id`.
Each request opens a transaction, sets the verified carrier context and relies on both RBAC
and PostgreSQL RLS. System-administration access is explicit and audited.

The target administration client is Leptos. The target mobile client is Flutter with a small,
audited `flutter_rust_bridge` surface for security-sensitive shared logic. Expo remains a UX
reference and maintenance client until the Flutter acceptance gates pass.

## Data and cutover strategy

Schema changes follow expand/migrate/contract:

1. add backward-compatible tables/columns and policies;
2. deploy code that can read old and new forms;
3. backfill with counted, restartable jobs and reconciliation reports;
4. shadow-read or dual-write only where an issue defines ownership and failure handling;
5. route one capability or tenant cohort to the target implementation;
6. verify functional, isolation and observability gates;
7. remove the old path only after a rollback window.

Database backups and a tested restore are required before destructive migration steps. No
big-bang schema or traffic cutover is allowed.

## Runtime and deployment

- Local development and disposable integration databases may use Docker Compose.
- Production uses immutable images promoted through environments and deployed to the OVH VPS.
- The reverse proxy can route capability slices during migration; a release must retain an
  explicit rollback target.
- Logs, traces and metrics must redact secrets and personal data.

## Security baseline

- Short-lived access tokens, rotated/revocable refresh tokens and MFA for privileged roles.
- Stable authorization roles: `system_admin`, `carrier_admin`, `driver`, `controller`,
  `chaperone`, `guardian`, `customer`.
- Idempotency for payments, scans and event ingestion.
- Signed, versioned, PII-minimal ticket presentations with key rotation.
- Audit records for privileged reads, writes and failed validation attempts.

Accepted architectural decisions live in `docs/adr/`. Later ADRs supersede this overview only
when they state the relationship explicitly.
