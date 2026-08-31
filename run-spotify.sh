#!/bin/bash
# Spotify + Spicetify dans podman, streamé via Selkies (https://localhost:8031)
# Session/config Spotify persistées dans ./data
DATA_DIR="$(cd "$(dirname "$0")" && pwd)/data"

podman run -d --replace --name spotify \
    -p 8030:3000 -p 8031:3001 \
    -e PUID=1000 -e PGID=1000 \
    -e TZ="$(cat /etc/timezone 2>/dev/null || echo Europe/Paris)" \
    -e CUSTOM_USER=spotify -e PASSWORD=spotify \
    --device /dev/dri \
    --shm-size=1g \
    -v "$DATA_DIR":/config \
    localhost/spicetify-web

echo "Spotify démarré : https://localhost:8031 (ou http://localhost:8030)"
podman logs -f spotify
