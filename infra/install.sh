#!/usr/bin/env bash
# RoadRunner installer — takes a fresh Ubuntu/Debian VPS to a running, HTTPS-secured RoadRunner
# instance. Designed to be safe to re-run: the first run bootstraps everything (Docker, the repo
# checkout, a generated .env.production), and every later run just pulls the latest image and
# redeploys. This is the ONE script both a customer's own sysadmin and our GitHub Actions deploy
# workflow use — there is deliberately no separate "first-time setup" vs "redeploy" script to
# keep in sync.
#
# Requires: root, or a user with passwordless sudo (both are standard for a dedicated deploy
# user / an SSH deploy keypair — see .github/workflows/deploy.yml's header comment).
#
# Usage (interactive, on the target server):
#   curl -fsSL https://raw.githubusercontent.com/uzzysan/roadrunner/main/infra/install.sh | bash
#
# Usage (non-interactive, e.g. CI): export DOMAIN and ACME_EMAIL first, then run this script —
# it skips the prompts when both are already set in the environment.
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/uzzysan/roadrunner.git}"
INSTALL_DIR="${INSTALL_DIR:-/opt/roadrunner}"

if [ "$(id -u)" = "0" ]; then
  SUDO=""
else
  SUDO="sudo"
fi

echo "==> RoadRunner installer"

# --- 1. Docker + Compose plugin -------------------------------------------------------------
if ! command -v docker >/dev/null 2>&1; then
  echo "==> Docker not found — installing via the official convenience script"
  curl -fsSL https://get.docker.com | $SUDO sh
  $SUDO systemctl enable --now docker
else
  echo "==> Docker already installed ($(docker --version))"
fi

if ! $SUDO docker compose version >/dev/null 2>&1; then
  echo "Docker Compose plugin not found even after Docker install." >&2
  echo "See https://docs.docker.com/compose/install/linux/ and re-run this script." >&2
  exit 1
fi

# --- 2. Repo checkout -------------------------------------------------------------------------
if [ -d "$INSTALL_DIR/.git" ]; then
  echo "==> $INSTALL_DIR already exists — pulling latest"
  git -C "$INSTALL_DIR" pull --ff-only origin main
else
  echo "==> Cloning $REPO_URL into $INSTALL_DIR"
  $SUDO mkdir -p "$INSTALL_DIR"
  $SUDO chown "$(id -u):$(id -g)" "$INSTALL_DIR"
  git clone "$REPO_URL" "$INSTALL_DIR"
fi
cd "$INSTALL_DIR"

# --- 3. Configuration -------------------------------------------------------------------------
if [ ! -f .env.production ]; then
  echo "==> First-time setup: generating .env.production"
  if [ -z "${DOMAIN:-}" ] || [ -z "${ACME_EMAIL:-}" ]; then
    if [ -t 0 ]; then
      read -rp "Domain this server will be reachable at (e.g. transit.acme-buses.com): " DOMAIN
      read -rp "Admin email for TLS certificate notices: " ACME_EMAIL
    else
      echo "DOMAIN and ACME_EMAIL must be set in the environment for a non-interactive run." >&2
      exit 1
    fi
  fi
  DB_PASSWORD="$(openssl rand -hex 24)"
  JWT_SECRET="$(openssl rand -hex 32)"
  cat > .env.production <<EOF
DOMAIN=${DOMAIN}
ACME_EMAIL=${ACME_EMAIL}
DB_USER=roadrunner
DB_PASSWORD=${DB_PASSWORD}
DB_NAME=roadrunner
DATABASE_URL=postgres://roadrunner:${DB_PASSWORD}@postgres:5432/roadrunner
JWT_SECRET=${JWT_SECRET}
JWT_EXPIRATION=86400
PORT=3000
HOST=0.0.0.0
RUST_LOG=info
EOF
  chmod 600 .env.production
  echo "    Wrote .env.production (DB password and JWT secret generated randomly — back this"
  echo "    file up somewhere safe; losing it means losing access to the database)."
else
  echo "==> .env.production already exists — leaving it as-is"
fi

# --- 4. Pull ghcr.io image ----------------------------------------------------------------
# Only needed if ghcr.io/uzzysan/roadrunner is private; harmless no-op if already logged in or
# the package is public. Uses a token so this script never needs an interactive password prompt.
if [ -n "${GHCR_USER:-}" ] && [ -n "${GHCR_TOKEN:-}" ]; then
  echo "${GHCR_TOKEN}" | $SUDO docker login ghcr.io -u "${GHCR_USER}" --password-stdin
fi

# --- 5. Bring the stack up -------------------------------------------------------------------
echo "==> Pulling latest image and starting the stack"
$SUDO docker compose -f infra/docker-compose.prod.yml --env-file .env.production pull
$SUDO docker compose -f infra/docker-compose.prod.yml --env-file .env.production up -d
$SUDO docker image prune -f >/dev/null 2>&1 || true

echo "==> Done. Current status:"
$SUDO docker compose -f infra/docker-compose.prod.yml ps

DOMAIN_OUT="$(grep -m1 '^DOMAIN=' .env.production | cut -d= -f2-)"
echo ""
echo "RoadRunner should be reachable at https://${DOMAIN_OUT} within a minute or two, once Caddy"
echo "obtains its TLS certificate (make sure the domain's DNS A record already points at this"
echo "server's IP, or Caddy won't be able to complete the ACME challenge)."
