# RoadRunner context

RoadRunner is a multi-tenant platform for local public transport and school-transport safety.
This repository contains both the deployed MVP baseline and the code that will replace it.

## Source of truth

- Canonical repository: <https://github.com/uzzysan/roadrunner>
- Work tracking: the Backend and Frontend RoadRunner projects in Linear
- Architecture: `docs/architecture.md`
- Delivery order: `docs/development-plan.md`
- Current status: `docs/status-log.md`
- UI rules: `docs/design-language.md`

Historical files under `status/` and the uppercase planning documents remain evidence of
past work. They are not delivery authority when they conflict with the files listed above.

## Delivery tracks

- **Deployed MVP**: the current single-crate Axum/SQLx backend and React Native/Expo client.
  It receives security, privacy, data-integrity and availability fixes only.
- **Target rewrite**: the target multi-tenant Rust workspace, Leptos administration UI and
  Flutter mobile clients. It is delivered incrementally in this repository.

Every Linear issue must carry exactly one of the `Deployed MVP` or `Target rewrite` labels.
Cross-cutting governance work belongs to `Target rewrite` because it controls the migration.

## Domain vocabulary

- **Carrier**: a transport operator and the tenant boundary.
- **Customer**: a passenger account that can buy and present tickets.
- **Guardian**: an adult authorized to view a linked student's trip state after consent.
- **Student**: a child participating in the school-transport flow.
- **Driver**: a user assigned to a vehicle or trip and allowed to publish positions.
- **Controller**: a user allowed to validate tickets.
- **Chaperone**: a user allowed to check students in and out of a trip.
- **Trip**: one dated execution of a route and schedule.
- **Ticket**: a purchased travel entitlement; a presentation token is not the ticket identity.
- **Tenant-scoped data**: data owned by one carrier and isolated by application checks and RLS.

Avoid using “operator” for a Carrier because it is ambiguous with an operations user.

## Non-negotiable invariants

1. No real-user or payment data enters the deployed MVP before BCK-15 is complete.
2. Tenant-scoped reads and writes require explicit carrier context and PostgreSQL RLS.
3. Security-sensitive state transitions are transactional and idempotent.
4. UI work follows the canonical entry point in `docs/design-language.md`; visual tokens must
   be shared across clients rather than redefined per application.
5. Migrations are expand/migrate/contract and remain compatible with the running version
   until traffic has moved and rollback is no longer required.
