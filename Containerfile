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
    && apt-get clean && rm -rf /var/lib/apt/lists/*

ARG SPICETIFY_VERSION=2.44.0
RUN curl -fsSL -o /tmp/spicetify.tar.gz \
        https://github.com/spicetify/cli/releases/download/v${SPICETIFY_VERSION}/spicetify-${SPICETIFY_VERSION}-linux-amd64.tar.gz \
    && mkdir -p /opt/spicetify \
    && tar xzf /tmp/spicetify.tar.gz -C /opt/spicetify \
    && chmod +x /opt/spicetify/spicetify \
    && rm /tmp/spicetify.tar.gz \
    && ln -s /opt/spicetify/spicetify /usr/local/bin/spicetify

# Openbox autostart: clean stale Chromium/crashpad locks (silent crash otherwise),
# then launch Spotify inside a D-Bus session (no session bus = 100% CPU busy loop)
RUN printf '%s\n' \
        '#!/bin/bash' \
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
        'chown -R abc:abc /config/.cache /config/.config "$SPICETIFY_CONFIG" 2>/dev/null || true' \
        'rm -rf /config/.cache/spotify/pending' \
        'rm -f /config/.cache/spotify/Singleton*' \
        'mkdir -p /config/.config/spotify "$SPICETIFY_CONFIG/Extensions"' \
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
        'case " $SPICETIFY_EXTENSIONS " in' \
        '    *" adblock.js "*)' \
        '        [ -f "$SPICETIFY_CONFIG/Extensions/adblock.js" ] || curl -fsSL -o "$SPICETIFY_CONFIG/Extensions/adblock.js" https://raw.githubusercontent.com/rxri/spicetify-extensions/main/adblock/adblock.js || true' \
        '        ;;' \
        'esac' \
        'CHANGED=0' \
        'for kv in "extensions $WANT_EXT" "custom_apps $WANT_APPS" "theme $WANT_THEME"; do' \
        '    k=${kv%% *}; v=${kv#* }' \
        '    CUR=$(grep -E "^$k[[:space:]]*=" "$CFG" | sed "s/^$k[[:space:]]*=//; s/^[[:space:]]*//" || true)' \
        '    if [ "$CUR" != "$v" ]; then' \
        '        sed -i "s~^$k[[:space:]]*=.*~$k               = $v~" "$CFG"' \
        '        CHANGED=1' \
        '    fi' \
        'done' \
        'chown -R abc:abc "$SPICETIFY_CONFIG" 2>/dev/null || true' \
        'if [ "$CHANGED" = 1 ]; then' \
        '    su abc -s /bin/bash -c "HOME=$SPICETIFY_HOME SPICETIFY_CONFIG=$SPICETIFY_CONFIG spicetify apply" || true' \
        'fi' \
        > /custom-cont-init.d/50-spicetify-init.sh \
    && chmod +x /custom-cont-init.d/50-spicetify-init.sh
