#!/bin/bash
# Spotify + Spicetify dans podman, streamé via Selkies (https://localhost:8031)
# Session/config Spotify persistées dans ./data
#
# Plugins spicetify via env vars (listes séparées par des espaces) :
#   SPICETIFY_EXTENSIONS  (défaut: "adblock.js", téléchargé automatiquement)
#   SPICETIFY_CUSTOM_APPS (défaut: aucun, ex: "lyrics-plus new-releases")
#   SPICETIFY_THEME       (défaut: aucun)
DATA_DIR="$(cd "$(dirname "$0")" && pwd)/data"

podman run -d --replace --name spotify \
    -p 8030:3000 -p 8031:3001 -p 8032:8000 \
    -e PUID=1000 -e PGID=1000 \
    -e TZ="$(cat /etc/timezone 2>/dev/null || echo Europe/Paris)" \
    -e CUSTOM_USER=spotify -e PASSWORD=spotify \
    -e SPICETIFY_EXTENSIONS="${SPICETIFY_EXTENSIONS:-adblock.js}" \
    -e SPICETIFY_CUSTOM_APPS="${SPICETIFY_CUSTOM_APPS:-}" \
    -e SPICETIFY_THEME="${SPICETIFY_THEME:-}" \
    --device /dev/dri \
    --shm-size=1g \
    -v "$DATA_DIR":/config \
    localhost/spicetify-web

echo "Spotify démarré : https://localhost:8031 (ou http://localhost:8030)"
podman logs -f spotify
