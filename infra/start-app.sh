#!/usr/bin/env bash
# Starts (or restarts) the RoadRunner containers without pulling anything new — mirrors
# /opt/fairpact/start-app.sh's role for the fairpact app on the same host. Use this after a
# host reboot or manual container stop; use deploy.sh to actually roll out a new build.
set -euo pipefail

cd /opt/roadrunner
podman-compose -f infra/podman-compose.prod.yml up -d
podman ps --filter "name=roadrunner"
