# Deployment Notes

> **2026-08-24:** production target is now GitHub Actions → OVH VPS via Podman, mirroring the
> `fairpact` app's convention on that host — see `infra/deploy.sh`,
> `infra/podman-compose.prod.yml`, `.github/workflows/deploy.yml`. Coolify is no longer used.
> The Raspberry Pi 5 section below stays as a record of the earlier dev setup.

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
