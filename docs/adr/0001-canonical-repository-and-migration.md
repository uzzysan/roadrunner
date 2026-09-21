# ADR-0001: one canonical repository and incremental migration

- Status: Accepted
- Date: 2026-09-21
- Review date: 2026-12-21, or earlier if a separate rewrite repository is proposed
- Decision owners: RoadRunner Backend and Frontend projects

## Context

Planning referred to a deployed MVP repository and a separate multi-crate rewrite checkout.
The deployed repository has a real GitHub origin, CI and deployment history. The former local
rewrite path has no current working tree or remote; only a stale code index remains. Linear and
README links therefore pointed contributors at documentation that did not exist in their
checkout and allowed legacy implementation to be mistaken for target completion.

## Decision

`https://github.com/uzzysan/roadrunner` is the sole canonical repository for code,
architecture, migration history and delivery documentation.

The deployed MVP and target rewrite are delivery tracks, not repositories. Every Linear issue
is labeled `Deployed MVP` or `Target rewrite`. The MVP receives maintenance/security work only.
Target capabilities are introduced incrementally in the same repository and cut over using
expand/migrate/contract database changes and reversible traffic slices.

React Native/Expo is retained as the deployed and UX-reference client until Flutter reaches
its acceptance gates. Flutter is the target mobile client; Leptos is the target administration
client. The canonical design system and tokens are shared by all clients.

## Consequences

- Contributors can clone one URL and reach all authoritative documentation.
- Existing CI, deployment history and schema lineage are preserved.
- Target modules must coexist with legacy code during migration, so boundaries and ownership
  have to be explicit.
- Big-bang rewrites and destructive cutovers are prohibited.
- Historical plans remain available but are explicitly non-authoritative.

## Ownership and branch policy

- `main` is the releasable integration branch and must not receive direct feature pushes.
- Product changes use short-lived issue branches and pull requests named from their Linear ID.
- Pull requests require passing CI and review before merge; migrations additionally require a
  rollback/compatibility review.
- `.github/CODEOWNERS` is the enforceable ownership fallback for every path. More specific
  product-team ownership may be added when stable team handles exist.
- Deployed-MVP emergency fixes follow the same review path unless an active incident requires
  an expedited merge; the incident record must then capture the retrospective review.

## Supersedes

This decision supersedes documentation that names an unversioned local `RoadRunner` checkout
or a second repository as the source of truth, and historical React Native/Tauri end-state
decisions.
