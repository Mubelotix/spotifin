FROM ghcr.io/linuxserver/baseimage-selkies:debiantrixie

RUN apt-get update && apt-get install -y --no-install-recommends \
        curl ca-certificates \
    && mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://download.spotify.com/debian/pubkey_5384CE82BA52C83A.gpg \
        -o /etc/apt/keyrings/spotify-archive-keyring.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/spotify-archive-keyring.gpg] https://repository.spotify.com stable non-free" \
        > /etc/apt/sources.list.d/spotify.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        spotify-client \
        libasound2t64 \
        libayatana-appindicator3-1 \
        libgl1 \
        pulseaudio-utils \
        ffmpeg \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

ARG SPICETIFY_VERSION=2.44.0
RUN curl -fsSL -o /tmp/spicetify.tar.gz \
        https://github.com/spicetify/cli/releases/download/v${SPICETIFY_VERSION}/spicetify-${SPICETIFY_VERSION}-linux-amd64.tar.gz \
    && mkdir -p /opt/spicetify \
    && tar xzf /tmp/spicetify.tar.gz -C /opt/spicetify \
    && chmod +x /opt/spicetify/spicetify \
    && rm /tmp/spicetify.tar.gz \
    && ln -s /opt/spicetify/spicetify /usr/local/bin/spicetify

# Project extensions shipped with the image (installed into the spicetify
# config dir by the init script when listed in SPICETIFY_EXTENSIONS).
COPY bridge.js /opt/spicetify/Extensions/bridge.js

# Openbox autostart: clean stale Chromium/crashpad locks (silent crash otherwise),
# then launch Spotify inside a D-Bus session (no session bus = 100% CPU busy loop)
RUN printf '%s\n' \
        '#!/bin/bash' \
        'if mkdir /tmp/spotify-backend.lock 2>/dev/null; then ROCKET_ADDRESS=0.0.0.0 ROCKET_PORT=8000 AUDIO_DATA_DIR=/config/audio /usr/local/bin/spotify-server >> /tmp/backend-live.log 2>&1 & fi' \
        'for _ in {1..10}; do if pgrep -u abc -x spotify >/dev/null 2>&1; then exit 0; fi; sleep 1; done' \
        'if ! mkdir /tmp/spotify-autostart.lock 2>/dev/null; then exit 0; fi' \
        'rm -rf /config/.cache/spotify/pending' \
        'rm -f /config/.cache/spotify/Singleton*' \
        'exec dbus-launch --exit-with-session spotify --no-sandbox --disable-dev-shm-usage' \
        > /defaults/autostart \
    && chmod +x /defaults/autostart

# Init script (runs as root at boot): fix /config ownership (root-owned leftovers
# prevent the Spotify window from ever mapping), clean locks, apply spicetify on first boot.
# Plugins are driven by the SPICETIFY_EXTENSIONS / SPICETIFY_CUSTOM_APPS / SPICETIFY_THEME
# env vars (space-separated lists). Spicetify refuses to run as root and files it
# touches must stay owned by abc.
RUN mkdir -p /custom-cont-init.d \
    && printf '%s\n' \
        '#!/bin/bash' \
        'set -e' \
        'SPICETIFY_HOME=/config' \
        'SPICETIFY_CONFIG=/config/spicetify' \
        'CFG="$SPICETIFY_CONFIG/config-xpui.ini"' \
        'setsid nohup sh -c "while true; do u=\$(stat -c %u /defaults 2>/dev/null); echo \"\$(date +%T) \$u\" >> /config/owner-boot.log; if [ \"\$u\" != \"\$(id -u abc)\" ]; then echo \"\$(date +%T) FIXING\" >> /config/owner-boot.log; lsiown -R abc:abc /defaults >> /config/owner-boot.log 2>&1 || true; fi; sleep 1; done" >/dev/null 2>&1 &' \
        'chown -R abc:abc /config/.cache /config/.config "$SPICETIFY_CONFIG" 2>/dev/null || true' \
        'rm -rf /config/.cache/spotify/pending' \
        'rm -f /config/.cache/spotify/Singleton*' \
        'mkdir -p /config/.config/spotify "$SPICETIFY_CONFIG/Extensions" /config/.config/openbox' \
        'cat /defaults/autostart > /config/.config/openbox/autostart' \
        'chown abc:abc /config/.config/openbox/autostart' \
        'mkdir -p /config/audio/hls' \
        'chown -R abc:abc /config/audio' \
        'setsid nohup su abc -s /bin/bash -c "until [ -S /defaults/native ]; do sleep 1; done; while true; do ffmpeg -hide_banner -loglevel error -y -f pulse -i default -map 0:a -ac 2 -ar 44100 -c:a aac -b:a 192k -f adts /config/audio/recording.aac; sleep 2; done" >/dev/null 2>&1 &' \
        'setsid nohup su abc -s /bin/bash -c "until [ -S /defaults/native ]; do sleep 1; done; while true; do ffmpeg -hide_banner -loglevel error -y -f pulse -i default -map 0:a -ac 2 -ar 44100 -c:a aac -b:a 192k -f hls -hls_time 6 -hls_list_size 0 -hls_playlist_type event -hls_segment_filename /config/audio/hls/segment-%06d.ts /config/audio/hls/main.m3u8; sleep 2; done" >/dev/null 2>&1 &' \
        '[ -f /config/.config/spotify/prefs ] || touch /config/.config/spotify/prefs' \
        'chmod -R a+rwX /usr/share/spotify' \
        'if [ ! -f "$CFG" ]; then' \
        '    su abc -s /bin/bash -c "HOME=$SPICETIFY_HOME SPICETIFY_CONFIG=$SPICETIFY_CONFIG spicetify backup apply" || true' \
        'fi' \
        '[ -f "$CFG" ] || exit 0' \
        'join() { local IFS="|"; echo "$*"; }' \
        'WANT_EXT=$(join $SPICETIFY_EXTENSIONS)' \
        'WANT_APPS=$(join $SPICETIFY_CUSTOM_APPS)' \
        'WANT_THEME="$SPICETIFY_THEME"' \
        'for ext in $SPICETIFY_EXTENSIONS; do' \
        '    case "$ext" in' \
        '        adblock.js)' \
        '            [ -f "$SPICETIFY_CONFIG/Extensions/adblock.js" ] || curl -fsSL -o "$SPICETIFY_CONFIG/Extensions/adblock.js" https://raw.githubusercontent.com/rxri/spicetify-extensions/main/adblock/adblock.js || true' \
        '            ;;' \
        '        *)' \
        '            if [ -f "/opt/spicetify/Extensions/$ext" ]; then cp "/opt/spicetify/Extensions/$ext" "$SPICETIFY_CONFIG/Extensions/$ext"; fi' \
        '            ;;' \
        '    esac' \
        'done' \
        'CHANGED=0' \
        'for kv in "extensions $WANT_EXT" "custom_apps $WANT_APPS" "theme $WANT_THEME"; do' \
        '    k=${kv%% *}; v=${kv#* }' \
        '    CUR=$(grep -E "^$k[[:space:]]*=" "$CFG" | sed "s/^$k[[:space:]]*=//; s/^[[:space:]]*//" || true)' \
        '    if [ "$CUR" != "$v" ]; then' \
        '        sed -i "s~^$k[[:space:]]*=.*~$k               = $v~" "$CFG"' \
        '        CHANGED=1' \
        '    fi' \
        'done' \
        'APPLIED_DIR=/usr/share/spotify/Apps/xpui/extensions' \
        'for ext in $SPICETIFY_EXTENSIONS; do if [ ! -f "$APPLIED_DIR/$ext" ]; then echo "$ext missing from applied app, forcing apply"; CHANGED=1; fi; done' \
        'chown -R abc:abc "$SPICETIFY_CONFIG" 2>/dev/null || true' \
        'if [ "$CHANGED" = 1 ]; then' \
        '    su abc -s /bin/bash -c "HOME=$SPICETIFY_HOME SPICETIFY_CONFIG=$SPICETIFY_CONFIG spicetify apply" || true' \
        'fi' \
        'pkill -u abc -x spotify 2>/dev/null || true' \
        > /custom-cont-init.d/50-spicetify-init.sh \
    && chmod +x /custom-cont-init.d/50-spicetify-init.sh

# Build the backend last so changes to the application do not invalidate the
# earlier Spotify and Spicetify setup layers.
WORKDIR /opt/spotify-mcp
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN apt-get update \
    && apt-get install -y --no-install-recommends cargo rustc build-essential pkg-config \
    && cargo build --release --locked \
    && install -m 0755 target/release/spotify-server /usr/local/bin/spotify-server \
    && apt-get purge -y cargo rustc build-essential pkg-config \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /usr/local/cargo/registry /usr/local/cargo/git /opt/spotify-mcp
