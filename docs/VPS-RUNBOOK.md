# RoadRunner — VPS deployment runbook

Written 2026-08-26. This is a handoff document for whoever (human or Claude Code) is operating
the `roadrunner-dev` practice VPS next. It explains the concept, the current architecture, exactly
what is deployed where, the open incident as of this writing, and what to keep an eye on going
forward.

## 1. Concept

RoadRunner is a multi-tenant platform for local public bus transport with an integrated school
transport safety module (see `docs/architecture.md` in the newer `roadrunner` design repo for the
full product architecture — buses, drivers, GPS tracking, ticketing, payments, student
safety/geofencing).

**The deployment requirement that shapes everything below:** RoadRunner is meant to be sold and
operated as software a transportation company installs on *its own* server — their own VPS, or
hardware they already run — not software permanently co-located on our infrastructure. That
server has no assumptions we can lean on: no existing reverse proxy, no pre-issued TLS
certificate, possibly nothing installed at all beyond a base OS. So "deployment" has to mean a
self-contained, one-command install that works unattended on a box nobody has hand-tuned, not a
bespoke fit to one host's layout.

The VPS this runbook covers — `217.182.74.51`, domain `roadrunner-dev.maculewicz.pro` — is a
disposable **practice server**, provided specifically so this install path could be exercised
against a real fresh box before it's treated as the real customer-facing install story. It is not
running any customer data and can be wiped and reinstalled at any time without consequence — treat
it that way when debugging (see §5).

## 2. Repositories — which one is deployed

There are two repos in play and it's easy to mix them up:

- **`github.com/uzzysan/roadrunner`** — the repo actually cloned and run on the VPS. This is the
  older, working single-crate Axum/SQLx MVP codebase (`src/main.rs`, `src/auth`, `src/handlers`,
  `src/models`, `src/tickets`, `src/payments`, `src/websocket`, `migrations/`). Everything in this
  runbook refers to this repo. Its `infra/` directory and `.github/workflows/deploy.yml` are the
  actual deployment machinery.
- The newer multi-crate workspace design (`crates/domain`, `crates/api`, `crates/gps-tracking`,
  etc., documented in a separate `roadrunner` repo's `docs/architecture.md`) is the target
  architecture for a future rewrite. It is **not** what's running on the VPS today. Don't confuse
  its `docs/architecture.md` §9 (which also describes this same Docker+Caddy deployment concept)
  with actual deployed code — it's the design doc, this runbook is the operational reality.

## 3. Architecture as deployed

```
Internet
   │  :80 / :443
   ▼
┌─────────────┐
│   Caddy     │  caddy:2-alpine — only container exposed to the internet.
│  (reverse   │  Auto-obtains/renews the Let's Encrypt cert for $DOMAIN via ACME.
│   proxy)    │  Reverse-proxies everything to api:3000 over the internal Docker network.
└──────┬──────┘
       │ internal docker network only, no published host port
       ▼
┌─────────────┐        ┌──────────────┐
│   api        │◄──────►│  postgres    │  postgis/postgis:16-3.4, also internal-only.
│ (roadrunner  │  5432   │  (PostGIS)   │  Healthcheck: pg_isready.
│  Rust binary)│        └──────────────┘
└─────────────┘
```

Three services, defined in `infra/docker-compose.prod.yml`:

- **`caddy`** — the only service with published ports (80/443). Config in `infra/Caddyfile`,
  templated with `{$DOMAIN}` / `{$ACME_EMAIL}` env vars. Handles HTTPS termination and gzip; adds
  HSTS/nosniff/frame-deny headers.
- **`api`** — the RoadRunner Rust binary, built from the root `Dockerfile` and published to
  `ghcr.io/uzzysan/roadrunner:latest` (private image). On startup it runs
  `sqlx::migrate!("./migrations")` against Postgres automatically — migrations are embedded into
  the binary at *compile* time, so there is no separate migration step, CLI, or copied SQL file
  needed at runtime, and this also means a bare (non-Docker) binary deploy gets migrations too.
  Binds `0.0.0.0:$PORT` (default 3000) unconditionally, regardless of the `HOST` env var.
- **`postgres`** — `postgis/postgis:16-3.4`. `api` waits for its healthcheck (`condition:
  service_healthy`) before starting.

Only `caddy` publishes host ports; `api` and `postgres` are reachable only over the internal
Docker network.

### The installer — `infra/install.sh`

This is the entire install story, and it's deliberately the *same* script for first-time bootstrap
and every later redeploy (no separate "provisioning" vs "redeploy" script to keep in sync):

1. Installs Docker + Compose plugin if missing (`get.docker.com` convenience script).
2. Clones `github.com/uzzysan/roadrunner` into `/opt/roadrunner` (or `git pull --ff-only` if
   already cloned).
3. On first run only: generates `/opt/roadrunner/.env.production` — prompts interactively for
   `DOMAIN`/`ACME_EMAIL` (or requires them pre-exported for non-interactive/CI runs), and
   auto-generates `DB_PASSWORD`/`JWT_SECRET` via `openssl rand -hex`. **Never touches
   `.env.production` again once it exists** — so redeploys never reset secrets.
4. If `GHCR_USER`/`GHCR_TOKEN` are set, logs into `ghcr.io` (needed because the image is private).
5. `docker compose -f infra/docker-compose.prod.yml --env-file .env.production pull`, then `up -d`,
   then prunes dangling images.
6. Prints final `docker compose ps` status and the expected `https://$DOMAIN` URL.

**Important gotcha baked into this script and worth remembering:** every single `docker compose
-f infra/docker-compose.prod.yml ...` invocation must include `--env-file .env.production`,
including read-only ones like `ps` — the compose file uses `${VAR:?error message}` required-value
interpolation for `DOMAIN`/`ACME_EMAIL`/`DB_PASSWORD`, and Compose fails to resolve those without
the env file explicitly passed on *every* invocation, not just the first one in a script. This
exact bug (missing `--env-file` on the trailing `ps` check) caused deploy run #4 to report failure
in GitHub Actions even though the underlying deploy had actually succeeded — fixed in commit
`b835022`. If you add any new `docker compose -f infra/docker-compose.prod.yml` line anywhere,
make sure `--env-file .env.production` is on it.

### CI/CD — `.github/workflows/`

- **`Build and Push Docker Image`** — builds the root `Dockerfile` and pushes
  `ghcr.io/uzzysan/roadrunner:latest` on every push to `main` that touches relevant files.
- **`Deploy to VPS`** (`deploy.yml`) — triggers automatically via `workflow_run` after the build
  workflow succeeds on `main`, or manually via `workflow_dispatch` (Actions tab → Deploy to VPS →
  Run workflow). SSHes into the VPS (`appleboy/ssh-action@v1.0.3`) using a dedicated deploy
  keypair and re-runs `install.sh` fetched fresh from `raw.githubusercontent.com`. There is no
  separate provisioning step — running `install.sh` against a box with nothing on it *is* the
  bootstrap.

Required GitHub repo secrets (Settings → Secrets and variables → Actions), all already set for
this VPS:

| Secret | Purpose |
|---|---|
| `VPS_HOST` | `217.182.74.51` |
| `VPS_SSH_USER` | the deploy user on the VPS |
| `VPS_SSH_KEY` | private half of a dedicated ed25519 deploy keypair (`roadrunner_deploy_key`) |
| `DOMAIN` | `roadrunner-dev.maculewicz.pro` |
| `ACME_EMAIL` | contact address for Let's Encrypt notices |
| `GHCR_USER` | `uzzysan` |
| `GHCR_TOKEN` | classic PAT, `read:packages` scope only — the image is private |

The matching public key is in the deploy user's `~/.ssh/authorized_keys` on the VPS, and the
private half also exists locally at `$HOME\.ssh\roadrunner_deploy_key` on the operator's own
machine (confirmed working for manual SSH access, independent of GitHub Actions).

## 4. Current status as of 2026-08-26 (open incident)

Timeline of the practice deploys, most recent first:

- **Deploy #6** (auto-triggered after `Build and Push Docker Image #36` finished building commit
  `b835022`) — GitHub Actions reported **success**, pulled the newest image, ran `up -d`.
- **Deploy #5** (manual `workflow_dispatch`, triggered right after commit `b835022` was pushed) —
  also reported success, but likely ran *before* the new image had finished building (image builds
  take ~7 minutes), so it probably redeployed with the previous image. Superseded by #6 seconds
  later in real terms but ~7 minutes later in the Actions log.
- **Deploy #4** — GitHub Actions reported failure, but the failure was cosmetic: all three
  containers actually came up and `postgres` reported healthy. The job only failed because of the
  missing `--env-file` bug described in §3, fixed by `b835022`.
- **Deploy #3** — genuine failure: `docker compose pull` got `unauthorized` from `ghcr.io`
  because the image is private and `GHCR_USER`/`GHCR_TOKEN` weren't set yet. Fixed by adding those
  two secrets.
- **Deploys #1–#2** — early runs, predate the current Docker+Caddy install path.

**Despite Deploy #6 reporting success, `https://roadrunner-dev.maculewicz.pro/` currently returns
HTTP 502 from Caddy.** A 502 from Caddy specifically means: DNS resolved, TLS handshake and
certificate are fine (Caddy answered the HTTPS request at all), but Caddy could not get a valid
response from `api:3000` on the internal network. That points at the `api` container, not at Caddy
or DNS/TLS.

**Leading hypothesis, not yet confirmed:** earlier deploy attempts (#1–#4) ran against an *older*
version of the Dockerfile/deploy pipeline, from before this session rewrote it, which may have
applied database migrations a different way (there was an earlier iteration that used
`sqlx-cli`/`sqlx migrate run` rather than the embedded `sqlx::migrate!()` macro now baked into the
binary — see the git history around commits `10c30df` and earlier). If Postgres already has a
`_sqlx_migrations` tracking table from that earlier path, and its recorded checksums/order don't
exactly match what's now embedded in the binary, `sqlx::migrate!(...).run(&pool).await.expect(...)`
in `src/main.rs` will panic on startup (`VersionMismatch`/`ChecksumMismatch` from the `sqlx`
migrate module), and the `api` container will crash-loop forever — which is exactly what a
persistent 502 from Caddy looks like from the outside. This has **not been confirmed** because no
one has yet read the actual `api` container logs — see §5 for how.

Other, less likely possibilities worth ruling out if the above isn't it: `api` still mid-restart
when checked (unlikely — should stabilize within seconds); a bad value in `.env.production` (it's
only generated once and never touched by redeploys, so an old/half-written value from a failed
early run could persist); the `postgres` healthcheck passing before Postgres is actually ready to
accept the app's connection pool.

## 5. What to do next (diagnostics)

SSH into the VPS with the existing deploy key (same access already confirmed working) and run:

```bash
cd /opt/roadrunner
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production ps
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production logs api --tail=100
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production logs postgres --tail=50
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production logs caddy --tail=50
```

Read the `api` log first — if it's crash-looping, the panic message from `sqlx::migrate!()` (or
whatever else) will be right there, printed once per restart.

### If it's the migration-checksum hypothesis (most likely)

Because this is a disposable practice VPS with no real data, the simplest fix is to wipe the
Postgres volume and let a clean set of migrations apply from scratch, rather than trying to hand-
reconcile a `_sqlx_migrations` table:

```bash
cd /opt/roadrunner
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production down
sudo docker volume rm roadrunner_roadrunner_pgdata   # confirm exact name with `docker volume ls` first
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production up -d
```

(Volume name is `<project>_roadrunner_pgdata` — Compose prefixes the volume name declared in
`docker-compose.prod.yml` with the project name, which defaults to the containing directory name,
`roadrunner`. Run `docker volume ls | grep pgdata` to get the exact name before removing anything.)

**Do not do this against a VPS holding real customer data** — this only holds true because the
practice VPS is explicitly disposable. For a real deploy, a checksum mismatch would need a proper
migration reconciliation, not a wipe.

### If it's something else

Whatever the `api` log shows, cross-reference against `src/config.rs` (env vars read, with their
defaults/fallbacks — only `DATABASE_URL` has no fallback and will panic if unset) and
`.env.production` on the box (`sudo cat /opt/roadrunner/.env.production` — contains secrets, don't
paste it anywhere public) to check for a missing or malformed value.

### Confirming the fix

Once `api`'s log shows it bound to `0.0.0.0:3000` and stayed up (no restart loop in `docker compose
ps` — `STATUS` should read `Up X seconds/minutes`, not cycling), reload
`https://roadrunner-dev.maculewicz.pro/health` — should return the app's health-check response, not
a Caddy 502.

## 6. Ongoing things to look after

- **This VPS is a practice environment, not production.** Feel free to `down -v` / wipe / reinstall
  from scratch here while validating the install path — that's its entire purpose. Do not port
  that habit to a real customer's server without the reconciliation caveat in §5.
- **The `ghcr.io/uzzysan/roadrunner` image is private** and tagged only `:latest` — there is
  currently no versioned/pinned tag and therefore no rollback mechanism beyond re-running
  `install.sh` against a specific past commit's built image manually. Worth adding tag-per-commit
  pushes (e.g. `:sha-<short-sha>` alongside `:latest`) before this becomes the real customer
  install path, so a bad deploy can be rolled back by pinning a tag instead of reverting and
  rebuilding.
- **No automated Postgres backup exists yet.** `roadrunner_pgdata` is a plain Docker volume with no
  snapshot/export job. Not urgent for the practice VPS, but must exist before any real customer
  data lands on a deployment built this way.
- **`GHCR_TOKEN` is a classic PAT** — check its expiry periodically (classic PATs can be set to
  never expire, but confirm this one's setting) and rotate if it's ever compromised; scope is
  `read:packages` only, so a leak only exposes pull access to the (already-private-for-a-reason)
  image, not push/write access.
- **Caddy's TLS renewal is automatic** (ACME/Let's Encrypt via the `caddy_data` volume) — nothing
  to do here unless the volume is deleted, in which case Caddy just re-obtains a cert on next
  start, provided DNS still points at the box.
- **fail2ban / SSH hygiene**: avoid repeated failed password-based SSH attempts against this or any
  OVH VPS — noted previously as a lockout risk. Key-based access (the deploy key, or the operator's
  personal key) doesn't trigger this.
- **Every `docker compose -f infra/docker-compose.prod.yml` command needs `--env-file
  .env.production`** — see §3. This is the single easiest mistake to reintroduce when editing
  `install.sh` or running ad-hoc commands by hand on the box.
- **`.env.production` is only ever written once**, on first bootstrap. If you need to change
  `DOMAIN`, rotate `DB_PASSWORD`, or fix a bad value, it must be edited by hand on the VPS (or
  deleted so `install.sh` regenerates it — which will also generate a *new* `DB_PASSWORD`, orphaning
  the existing Postgres volume's credentials unless you update them to match, or wipe the volume
  too).
- **Both `docs/DEPLOY.md` (Polish, historical Raspberry Pi 5 notes) and `docs/DEPLOYMENT.md`
  (English, same) carry pointer notes to this Docker+Caddy approach** — keep this runbook as the
  single source of truth for *this* deployment's operational status; update those pointers if the
  deployment story changes again.

## 7. Quick reference

| What | Value |
|---|---|
| VPS IP | `217.182.74.51` |
| Domain | `roadrunner-dev.maculewicz.pro` |
| Install dir on VPS | `/opt/roadrunner` |
| Repo | `github.com/uzzysan/roadrunner` (branch `main`) |
| Image | `ghcr.io/uzzysan/roadrunner:latest` (private) |
| Compose file | `infra/docker-compose.prod.yml` |
| Env file (generated, secrets) | `/opt/roadrunner/.env.production` |
| Install/redeploy script | `infra/install.sh` |
| Deploy workflow | `.github/workflows/deploy.yml` |
| Manual redeploy | Actions tab → "Deploy to VPS" → Run workflow (branch `main`) |
