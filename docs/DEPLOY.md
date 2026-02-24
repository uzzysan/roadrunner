# Deployment RoadRunner

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
