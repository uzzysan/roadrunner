# Deployment Notes

> **2026-08-24:** production deployment is now a self-contained Docker + Caddy stack, installed by
> a single script (`infra/install.sh`) designed to run on ANY dedicated VPS a customer hands
> over — not tied to our own OVH host's existing setup. See `infra/docker-compose.prod.yml`,
> `infra/Caddyfile`, `infra/install.sh`, `.github/workflows/deploy.yml`. (An earlier revision of
> this plan mirrored the `fairpact` app's Podman setup on our shared OVH VPS — abandoned once the
> real requirement became "any transportation company's own server," which that approach didn't
> generalize to.) Coolify is not used at all. The Raspberry Pi 5 section below stays as a record
> of the earlier dev setup.

## Container Runtime

### Development (Raspberry Pi 5)
- **Runtime**: Podman (rootless)
- **Compose**: podman-compose
- **Note**: Docker commands need translation to podman equivalents

### Production
- **Runtime**: Docker
- **Compose**: docker-compose

## Podman vs Docker Commands

| Docker | Podman |
|--------|--------|
|  |  |
|  |  |
|  |  |
|  |  |

## PostgreSQL on Pi5 (Native)
Currently running native PostgreSQL 17 + PostGIS 3.5:
- User: roadrunner
- Database: roadrunner
- PostGIS: enabled

## AI Tools on Pi5

### kimi-cli
Installed and available for complex tasks on the remote machine.
Useful for:
- Code generation
- Documentation
- Complex refactoring
- Analysis tasks

Usage: kimi-cli <prompt>
