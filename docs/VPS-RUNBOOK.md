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

## 4. Incident of 2026-08-26 — RESOLVED (self-recovered before this write-up)

**Status as of this edit: the site is up.** `https://roadrunner-dev.maculewicz.pro/health` returns
`200 OK`, and `POST /auth/login` with a bogus credential pair correctly returns `401 {"error":
"Invalid credentials", ...}` — proving the request path all the way through to a Postgres query
works, not just that the process is alive. All three containers are up, `api` and `postgres` report
`(healthy)`. **This was not fixed by an operator action in this session — by the time it was
investigated, it had already recovered.** Below is the reconstruction of what happened, since the
original panic/crash logs could not be recovered (see caveat at the end).

Timeline of the practice deploys, most recent first:

- **Deploy #6** (auto-triggered after `Build and Push Docker Image #36` finished building commit
  `b835022`) — GitHub Actions reported **success**, pulled the newest image, ran `up -d`. The
  `ghcr.io/uzzysan/roadrunner:latest` image's `Created` timestamp is `2026-08-26T06:30:45Z`, and the
  `api` container's `StartedAt` is `2026-08-26T06:31:10Z` — 25 seconds later. That pull-and-recreate
  is what ended the incident (see below).
- **Deploy #5** (manual `workflow_dispatch`, triggered right after commit `b835022` was pushed) —
  also reported success, but likely ran *before* the new image had finished building (image builds
  take ~7 minutes), so it probably redeployed with the previous (still-broken) image.
- **Deploy #4** — GitHub Actions reported failure, but the failure was cosmetic: all three
  containers actually came up and `postgres` reported healthy. The job only failed because of the
  missing `--env-file` bug described in §3, fixed by `b835022`.
- **Deploy #3** — genuine failure: `docker compose pull` got `unauthorized` from `ghcr.io`
  because the image is private and `GHCR_USER`/`GHCR_TOKEN` weren't set yet. Fixed by adding those
  two secrets.
- **Deploys #1–#2** — early runs, predate the current Docker+Caddy install path.

### What the 502 actually was

A 502 from Caddy means DNS resolved, TLS was fine — Caddy just couldn't get a response from
`api:3000`. Reading Caddy's own error log for the incident window (`21:51` to `06:08` UTC — nearly
8 hours) showed every single request failing the same way, and *not* with the "connection refused"
you'd expect from a container that's merely slow to start:

```
dial tcp: lookup api on 127.0.0.11:53: server misbehaving
```

`127.0.0.11` is Docker's embedded per-container DNS resolver. It only resolves a service name to an
IP while that container is actually running — this error means the `api` container was **down**,
not merely unready, for essentially the entire window. Cross-checking `journalctl -u docker` for
that window shows `sbJoin` (network-endpoint-join) events for `roadrunner-api` firing roughly every
60–61 seconds, non-stop, for the full ~8 hours. That cadence is the signature of a genuine crash
loop: dockerd's restart backoff for `restart: unless-stopped` starts short and doubles up to a
**cap of 60 seconds**, so a container that keeps dying immediately after start settles into
retrying exactly once a minute. That matches perfectly — this was `api` crashing on every start,
not a one-off slow boot.

### The migration-checksum hypothesis (previous leading theory) is RULED OUT

The original version of this section suspected `sqlx::migrate!()` panicking on a checksum/version
mismatch against a pre-existing `_sqlx_migrations` table from an older deploy pipeline. Once `api`
finally logged a clean boot at `06:31:11`, its own migration run disproved this:

```
"SELECT version, checksum FROM _sqlx_migrations ORDER BY version"  rows_returned=0
```

Zero rows. The table was empty when this run started, and it then applied migration #1
(`CREATE TABLE users ...`) and every migration after it from scratch, with no unique-constraint
conflicts. If an earlier attempt had ever gotten far enough to record even the first migration,
this query would have returned at least one row. It didn't — meaning **no previous crash-looping
attempt ever got as far as committing a migration**, so there was nothing for a new binary to
disagree with. Whatever was crashing, it was dying earlier than that.

Postgres's own log for the entire incident window was also checked and contains **no** `FATAL`,
authentication, or connection-error lines at all (only 2 log lines total in ~9 hours — the
`postgis/postgis` image logs very little by default). That's consistent with `api` never
successfully completing a DB handshake during the crash loop, though it isn't fully conclusive on
its own since this image doesn't log `log_connections` by default.

### Unresolved: the exact panic message is gone

The container that was crash-looping got **replaced**, not merely restarted, when Deploy #6 pulled
the new image at `06:30:45` — `docker inspect roadrunner-api` shows `RestartCount=0` and
`Created == StartedAt`, i.e. this is a fresh container, and Docker does not retain a removed
container's logs. So the actual panic/error text that was printed on every one of those ~480
crash-loop attempts is unrecoverable. No crontab, systemd timer, or shell history on the box shows
anyone/anything manually intervening between `21:51` and `06:31` — the only thing that changed was
the new image landing via the normal CI pipeline. It's plausible (not proven) that the same class
of code change that produced commit `b835022` also touched something in the image-relevant paths
enough to trigger a rebuild that happened to fix whatever was crashing — but that can't be
confirmed after the fact. **If this recurs, capture `docker compose logs api` (or better, `docker
logs <container-id>` before doing anything else) before touching the stack** — see the addition to
§6 about durable logging.

## 5. Diagnostics reference (kept for the next incident)

SSH into the VPS with the existing deploy key (same access confirmed working) and run:

```bash
cd /opt/roadrunner
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production ps
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production logs api --tail=200
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production logs postgres --tail=100
sudo docker compose -f infra/docker-compose.prod.yml --env-file .env.production logs caddy --tail=50
```

Also useful, learned from this incident:

- `sudo docker inspect roadrunner-api --format 'RestartCount={{.RestartCount}} StartedAt={{.State.StartedAt}}'`
  — a high, climbing `RestartCount` on the *same* container ID confirms an active crash loop in
  real time (checked repeatedly a few seconds apart).
- `sudo journalctl -u docker --since '<window start>' --until '<window end>' | grep 'roadrunner-api.*sbJoin'`
  — a steady ~60s cadence of network-join events is the crash-loop signature even after the
  crashing container itself has been replaced and its logs lost.
- Caddy's own log (`logs caddy`) names the exact failure mode in `msg` — `"dial tcp: lookup api on
  127.0.0.11:53: server misbehaving"` means the `api` container isn't running at all (crash loop or
  fully down), as opposed to a `connection refused`, which would mean it's running but not yet
  listening.
- Don't restart, recreate, or `up -d` anything just to "double check" once the site is confirmed
  healthy — that discards the only container instance whose logs prove it booted cleanly, which is
  exactly the evidence this incident lost.

### If a migration-checksum panic ever *is* confirmed in the logs

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
migration reconciliation, not a wipe. **Only reach for this once the logs actually show a
`VersionMismatch`/`ChecksumMismatch` panic** — this incident showed that hypothesis can be wrong,
and wiping the volume destroys evidence you may still need.

### If it's something else

Whatever the `api` log shows, cross-reference against `src/config.rs` (env vars read, with their
defaults/fallbacks — only `DATABASE_URL` has no fallback and will panic if unset) and
`.env.production` on the box (`sudo cat /opt/roadrunner/.env.production` — contains secrets, don't
paste it anywhere public) to check for a missing or malformed value.

### Confirming a fix

Once `api`'s log shows it bound to `0.0.0.0:3000` and stayed up (no restart loop in `docker compose
ps` — `STATUS` should read `Up X seconds/minutes`, not cycling), reload
`https://roadrunner-dev.maculewicz.pro/health` — should return `OK`, not a Caddy 502. `/health` is a
static handler (`src/main.rs`, `health_check()`) and doesn't touch the database, so it only proves
the HTTP listener is up. To confirm the DB path specifically, hit an endpoint that queries Postgres,
e.g. `POST /auth/login` with a bogus email/password — a clean `401 {"error": "Invalid
credentials", ...}` (not a 500/502) proves the full request→pool→query→response path works.

## 6. Ongoing things to look after

- **Crash-loop logs are currently unrecoverable once `docker compose up -d` recreates the
  container** — the 2026-08-26 incident's root cause couldn't be confirmed for exactly this reason
  (see §4). `docker`'s default `json-file` log driver keeps logs per container *ID*; a redeploy that
  pulls a new image recreates the container (new ID) and the old logs are gone with it. Worth adding
  either a bumped `max-size`/`max-file` on the `json-file` driver so more history survives routine
  restarts, or forwarding container logs to a file on the host (e.g. a `local` logging driver
  writing under `/var/log/`, or a lightweight log-shipping sidecar) so a crash loop can be diagnosed
  after the fact instead of only in the moment it's happening.
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
