# RoadRunner development plan

Status: active delivery order, updated 2026-09-21. Linear owns task status; this document owns
the dependency and release policy.

## Track A: deployed MVP maintenance

Complete BCK-15 and its children before any real-user data is accepted:

1. BCK-20 — verified and idempotent Stripe webhooks;
2. BCK-21 — controller-only, atomic ticket validation;
3. BCK-19, BCK-22 and BCK-23 — WebSocket auth, abuse controls and session hardening;
4. FND-31 — device verification of the connected Expo reference client.

Only security, privacy, data-integrity and availability work belongs on this track.

## Track B: target rewrite

1. Governance: BCK-14 and BCK-16–18.
2. Phase 1 backend: BCK-24 tenant model/RLS, BCK-25 auth/RBAC, BCK-26 CRUD,
   BCK-27 passenger contract and BCK-28 disposable PostGIS test harness.
3. Phase 1 frontend: FND-14–16 after the corresponding backend contracts are stable.
4. Phase 2 GPS: BCK-29–32, then FND-17–18.
5. Phase 3 ticketing/payments: BCK-33–36, then FND-19 and FND-22.
6. Phase 4 school safety: BCK-35, BCK-37–38, then FND-20–21.
7. Flutter foundation and role modes: FND-23–27 after their backend capabilities.
8. Analytics and operations: BCK-39–40 and BCK-44, then FND-26 and FND-29.
9. Hardening and release: BCK-41–43 and FND-28–30.

Phases 2 and 3 may run in parallel once Phase 1 tenant isolation, auth context and test harness
are complete. Frontend work may start from a versioned mock contract but cannot be accepted
before integration against the real backend.

## Definition of ready

An issue is ready when it has one delivery-track label, unambiguous dependencies, acceptance
criteria, a named API/schema contract where relevant, and no unresolved product decision that
changes its security or data model.

## Definition of done

- Acceptance criteria and relevant negative paths are tested.
- Tenant isolation, authorization and idempotency are tested where applicable.
- Database changes migrate from zero and from the last released schema.
- CI passes format, lint, unit and integration gates.
- User-facing work satisfies `docs/design-language.md` and PL/EN completeness.
- The Linear issue records verification evidence and any intentionally deferred gap.
- Production-impacting work has monitoring and rollback instructions.

## PostgreSQL test baseline

Integration tests use a disposable `postgis/postgis:16-3.4` database, migrate from zero and
seed deterministic data for at least two carriers. Tests never depend on a developer database.
`BCK-28` owns the one-command local/CI harness and cross-tenant leakage suite.
