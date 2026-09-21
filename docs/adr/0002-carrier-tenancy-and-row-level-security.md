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
transaction. Application code must derive either `Carrier(carrier_id)` or `SystemAdmin` from
an authenticated session and call `TenantContext` before executing tenant queries. Context is
transaction-local so a pooled connection cannot retain the previous request's carrier.

Each tenant table enables and forces RLS with distinct SELECT, INSERT, UPDATE and DELETE
policies. Missing context matches no tenant row. Carrier context can only read and mutate rows
whose `carrier_id` matches. Explicit system-admin context can operate across carriers. Views
over tenant tables run as `security_invoker` so their owners cannot bypass the underlying
policies.

The production application database role must be non-superuser and must not have
`BYPASSRLS`; PostgreSQL superusers bypass RLS even when it is forced. Schema migrations may
continue to run with a separate privileged owner role.

## Migration

Two deterministic test carriers are seeded. Existing single-tenant data and user memberships
are assigned to Test Carrier A as an expand/backfill step. New tenant rows default their
`carrier_id` from the transaction context and therefore fail closed when the context is
missing. Natural identifiers such as route number and vehicle registration become unique per
carrier.

## Consequences

- Database policy remains the final isolation boundary if an endpoint omits a tenant filter.
- Every tenant-aware request must use a transaction, including read-only requests.
- Login identity lookup stays global; authorization to tenant data comes from membership and
  the verified request context.
- The JWT/RBAC ticket must carry the verified carrier membership into `TenantContext` rather
  than accepting carrier identifiers from request payloads or headers.
- Integration tests must exercise RLS through a non-superuser, non-`BYPASSRLS` database role.
