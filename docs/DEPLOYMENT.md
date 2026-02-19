# Deployment Notes

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
