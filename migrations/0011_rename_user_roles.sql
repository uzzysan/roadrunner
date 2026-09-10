-- Migration: Reconcile user_role enum with the target tenant RBAC set (issue #9 / R02)
--
-- Old values:  passenger, driver, attendant, parent, admin
-- New values:  customer, driver, chaperone, guardian, carrier_admin, controller
--
-- ALTER TYPE … RENAME VALUE requires PostgreSQL 10+.
-- Each RENAME is idempotent only if run once — the migration runner guarantees this.

ALTER TYPE user_role RENAME VALUE 'passenger' TO 'customer';
ALTER TYPE user_role RENAME VALUE 'attendant' TO 'chaperone';
ALTER TYPE user_role RENAME VALUE 'parent'    TO 'guardian';
ALTER TYPE user_role RENAME VALUE 'admin'     TO 'carrier_admin';
-- 'driver' is unchanged
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'controller';

COMMENT ON TYPE user_role IS
  'Tenant RBAC roles: customer, driver, chaperone, guardian, carrier_admin, controller. '
  'system_admin is not a tenant role under the per-VPS deployment model (architecture.md §9).';