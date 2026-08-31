# spotify-mcp

Spotify + Spicetify running in a rootless podman container, streamed to the browser via LinuxServer's Selkies baseimage (no X11/PulseAudio bridging with the host).

## Files

Keep the file count minimal:

- `Containerfile` — everything container-related lives here. Inline any container config (openbox autostart, init scripts, wrappers) into the Containerfile with `RUN printf '%s\n' ...` instead of creating `root/` files (podman 4.9 does not support Dockerfile heredocs).
- `run-spotify.sh` — the only helper script: builds nothing, just runs the container.
- `TARGET.md` — implementation target and compatibility reference for the Jellyfin music API.

## Preferences

- Communicate in English for this project.
- Don't leave temp/debug files hanging around in the repo.
- LSIO baseimage specifics that matter (don't relearn these the hard way):
  - Custom init scripts go to `/custom-cont-init.d/*.sh`, app autostart to `/defaults/autostart`.
  - `/config` is the persistent volume; anything root-owned left there prevents the Spotify window from mapping (black screen) — the init script chowns it to `abc` at boot.
  - Stale `/config/.cache/spotify/Singleton*` and `pending/` locks make Spotify crash silently at startup — cleaned at every boot.
  - Spotify needs a D-Bus session (`dbus-launch`) or its main loop busy-spins at 100% CPU with the window never mapping.

## Usage

```bash
podman build -t spicetify-web .
./run-spotify.sh   # then open https://localhost:8031 (user: spotify, password: spotify)
```

## Spicetify plugins

Plugins are configured with env vars in `run-spotify.sh` (space-separated lists), applied by the init script at every boot:

- `SPICETIFY_EXTENSIONS` (default `adblock.js` — auto-downloaded from rxri/spicetify-extensions)
- `SPICETIFY_CUSTOM_APPS` (e.g. `lyrics-plus new-releases`)
- `SPICETIFY_THEME`

```bash
SPICETIFY_EXTENSIONS="adblock.js fullAppDisplay.js" SPICETIFY_CUSTOM_APPS="lyrics-plus" ./run-spotify.sh
```

The resulting config is persisted in `./data/spicetify/config-xpui.ini`. Manual CLI config also works but must run as abc with HOME=/config — spicetify refuses root, and root-owned output files cause the black-screen bug:

```bash
alias spicetify='podman exec -u abc -e HOME=/config -e SPICETIFY_CONFIG=/config/spicetify spotify spicetify'
```

Built-in extensions are in `/opt/spicetify/Extensions/`, custom apps in `/opt/spicetify/CustomApps/` (lyrics-plus, new-releases, reddit). More via the spicetify Marketplace.

## Jellyfin API

`TARGET.md` documents the Jellyfin music API required for a compatible reimplementation. It is based on the Jellyfin server, Finamp, and jellyfin-audio-player sources cloned under `/tmp/target-research/`.
