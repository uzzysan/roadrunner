#!/usr/bin/env bash
# Deploys the latest RoadRunner image on the OVH VPS. Mirrors the convention already used by
# the `fairpact` app on the same host (/opt/fairpact/scripts/deploy.sh) — pull latest, restart
# via Podman Compose. Runs FROM /opt/roadrunner (this repo checked out there), invoked over SSH
# by .github/workflows/deploy.yml.
set -euo pipefail

cd /opt/roadrunner

echo "==> Pulling latest git history (for compose/migration file changes)"
git pull --ff-only origin main

echo "==> Pulling latest image"
podman pull ghcr.io/uzzysan/roadrunner:latest

echo "==> Recreating containers"
podman-compose -f infra/podman-compose.prod.yml up -d

echo "==> Pruning old images"
podman image prune -f

echo "==> Done. Recent container status:"
podman ps --filter "name=roadrunner"
