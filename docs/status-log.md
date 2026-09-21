# RoadRunner status log

This is the current, append-only project status. Detailed workflow state remains in Linear.

## 2026-09-21 — canonical repository and migration policy

- Canonical repo confirmed: <https://github.com/uzzysan/roadrunner>.
- The previously referenced rewrite checkout has no current working tree or remote; its stale
  local code index is not a source of truth.
- Target architecture will be migrated incrementally in the canonical repo.
- Deployed Axum/SQLx + Expo MVP is limited to maintenance/security until replaced.
- React Native/Expo remains the deployed/reference client; Flutter is the target mobile client.
- Leptos is the target administration client; historical Tauri guidance is superseded.
- Docker Compose is allowed for local disposable PostgreSQL/PostGIS; production promotion and
  rollback remain container-runtime independent, with OVH as the current deployment target.
- BCK-14 is implemented and submitted for review; BCK-18 retains the detailed rehearsal work.
- BCK-20 and BCK-21 are in review.
- Next target-rewrite step after BCK-14: BCK-24 (carrier context and PostgreSQL RLS), followed
  by BCK-28 once the tenant schema exists (disposable PostGIS integration-test harness).
- No real-user data is allowed before BCK-15 is complete.

## Historical records

`status.md`, `status/STATUS.md`, `status/DAILY_LOG.md` and uppercase planning documents are
retained as historical evidence. They must not be used to infer current scope or completion.
