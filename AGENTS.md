# spotify-mcp

Spotify + Spicetify running in a rootless podman container, streamed to the browser via LinuxServer's Selkies baseimage (no X11/PulseAudio bridging with the host).

## Files

Keep the file count minimal:

- `Containerfile` — everything container-related lives here. Inline any container config (openbox autostart, init scripts, wrappers) into the Containerfile with `RUN printf '%s\n' ...` instead of creating `root/` files (podman 4.9 does not support Dockerfile heredocs).
- `run-spotify.sh` — the only helper script: builds nothing, just runs the container.

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
