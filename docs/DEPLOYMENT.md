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

## AI Tools on Pi5

### kimi-cli
Installed and available for complex tasks on the remote machine.
Useful for:
- Code generation
- Documentation
- Complex refactoring
- Analysis tasks

Usage: kimi-cli <prompt>
