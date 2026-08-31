#!/bin/bash
# Spotify + Spicetify dans podman, streamé via Selkies (https://localhost:8031)
# Session/config Spotify persistées dans ./data
DATA_DIR="$(cd "$(dirname "$0")" && pwd)/data"

# Migration unique depuis l'ancien volume nommé
if [ -z "$(ls -A "$DATA_DIR" 2>/dev/null)" ] && podman volume exists spicetify-config 2>/dev/null; then
    echo "Migration de spicetify-config vers $DATA_DIR ..."
    mkdir -p "$DATA_DIR"
    podman run --rm -v spicetify-config:/from:ro -v "$DATA_DIR":/to alpine sh -c 'cp -a /from/. /to/'
    chown -R 1000:1000 "$DATA_DIR"
fi

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
