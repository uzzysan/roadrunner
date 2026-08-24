# Deployment RoadRunner

> **2026-08-24:** ten wpis dotyczy wcześniejszego, ręcznego deploymentu na Raspberry Pi 5.
> Docelowy production deployment to teraz GitHub Actions → OVH VPS (Podman) — patrz
> `infra/deploy.sh`, `infra/podman-compose.prod.yml`, `.github/workflows/deploy.yml` w tym repo.
> Ta strona zostaje jako zapis historyczny środowiska dev/testowego na Pi5.

## Status produkcji
- **Serwer**: http://100.83.82.95:3000
- **Health**: http://100.83.82.95:3000/health
- **WebSocket**: ws://100.83.82.95:3000/ws

## Zarządzanie serwisem
```bash
# Status
sudo systemctl status roadrunner

# Restart
sudo systemctl restart roadrunner

# Logi
sudo journalctl -u roadrunner -f
```

## Rebuild i redeploy
```bash
cd ~/Kodzenie/RoadRunner
git pull
cargo build --release
sudo systemctl restart roadrunner
```

## Konfiguracja
Plik: `.env.production`

| Zmienna | Opis |
|---------|------|
| DATABASE_URL | PostgreSQL connection string |
| JWT_SECRET | Secret key dla JWT (zmień!) |
| PORT | Port serwera (3000) |
| HOST | Bind address (0.0.0.0) |
