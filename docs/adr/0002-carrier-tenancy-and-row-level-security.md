# ADR-0002: carrier tenancy and PostgreSQL row-level security

- Status: Accepted
- Date: 2026-09-21
- Decision owners: RoadRunner Backend

## Context

The deployed schema was single-tenant. Operational rows had no carrier owner, so a missing
filter in any handler could expose or mutate another operator's data. Authentication users
also served as global login identities, which made adding a nullable `carrier_id` directly to
every user ambiguous for future system administrators and multi-carrier support.

## Decision

`carriers` is the global tenant registry. `users` remains the global authentication identity
directory and `carrier_memberships` links identities to carriers. Provider-wide Stripe event
deduplication and PostgreSQL/PostGIS/SQLx metadata also remain global. All transport,
ticketing, payment, incident and school-safety records are tenant-owned and carry a non-null
`carrier_id`.

Tenant access is installed with `SET LOCAL`-equivalent PostgreSQL settings inside a database
transaction. HTTP middleware verifies the selected active carrier and, for authenticated calls,
the signed JWT carrier plus current membership. `TenantPool` refuses unscoped SQL and wraps each
operation in a transaction that applies `TenantContext`; explicit multi-statement work uses the
same context on one transaction. Background paths such as the signed Stripe webhook must select
`SystemAdmin` explicitly. Context is transaction-local, so errors, cancellation and pooled
connection reuse cannot retain the previous request's carrier.

Each tenant table enables and forces RLS with distinct SELECT, INSERT, UPDATE and DELETE
policies. Missing context matches no tenant row. Carrier context can only read and mutate rows
whose `carrier_id` matches. Explicit system-admin context can operate across carriers. Views
over tenant tables run as `security_invoker` so their owners cannot bypass the underlying
policies.

The production application database role must be non-superuser and must not have
`BYPASSRLS`; PostgreSQL superusers bypass RLS even when it is forced. Schema migrations may
continue to run with a separate privileged owner role. Deployment creates a dedicated runtime
login that inherits the `roadrunner_app` grants, while startup rejects an unsafe runtime role.

Relationships between tenant-owned tables use composite foreign keys from
`(carrier_id, foreign_id)` to `(carrier_id, id)`. RLS controls row visibility and mutation;
these constraints separately prevent a valid row for one carrier from referencing or cascading
into another carrier's row.

WebSocket connections capture the verified request carrier before upgrade. Driver publishing
requires a signed Driver identity and an active database assignment to the vehicle; route and
vehicle subscriptions are checked through the same tenant transaction. GPS updates are persisted
under RLS and broadcast through a carrier-specific channel.

## Migration

Two deterministic test carriers are seeded. Existing single-tenant data and user memberships
are assigned to Test Carrier A as an expand/backfill step. New tenant rows default their
`carrier_id` from the transaction context and therefore fail closed when the context is
missing. Natural identifiers such as route number and vehicle registration become unique per
carrier.

## Consequences

- Database policy remains the final isolation boundary if an endpoint omits a tenant filter.
- Every tenant-aware database operation uses a transaction, including read-only operations.
- Login identity lookup stays global; authorization to tenant data comes from membership and
  the verified request context.
- JWTs carry the carrier selected at authentication time; later RBAC work may enrich roles but
  must preserve the membership re-check and must not trust carrier identifiers from payloads.
- Integration tests must exercise RLS through a non-superuser, non-`BYPASSRLS` database role.
