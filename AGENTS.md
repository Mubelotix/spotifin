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

- `SPICETIFY_EXTENSIONS` (default `adblock.js bridge.js`; `adblock.js` is auto-downloaded from rxri/spicetify-extensions, project extensions like `bridge.js` are COPY'd into `/opt/spicetify/Extensions/` at build time and installed by the init script)
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

`TARGET.md` documents the Jellyfin music API required for a compatible reimplementation. It is based on the Jellyfin server, Finamp, and jellyfin-audio-player sources cloned under `/tmp/target-research/`. A first implementation now exists in Rust (`src/jellyfin/`), see "Implementation decisions" below.

## Spicetify development docs

`spicetify-dev-docs.md` vendors the spicetify/docs development section (api-wrapper, Platform, CosmosAsync, GraphQL) as one greppable file; sections are delimited by `=== <path> ===` lines. It documents Spotify's internal client APIs, which `bridge.js` exposes through `/debug/eval`. Docs drift: verify everything against the live client before relying on it.

## Implementation decisions

- Ignore authentication. Spotify is assumed to already be connected.
- The instance has exactly one static user.
- Keep Spotify API queries to a minimum. Prefer clicking in the Spotify application and reading the resulting visual state, as this should be less detectable as bot activity. Make exceptions only when necessary.
- Playback progress is not important for now. Jellyfin progress may always be reported as zero; do not manage Spotify's playback cursor for this purpose.
- Spotify Daily playlists and all Spotify playlists visible on the Spotify home page should be available to Jellyfin clients. Prefer representing them as classic playlists if Instant Mix cannot provide multiple separate mixes.
- Lyrics are available through Spotify's interface using the microphone button.
- Audio is known to stream through the Selkies interface when Spotify is playing inside the container. The audio recording/streaming location is undecided: it may be in the Spicetify extension, in the Rust backend, or outside the container, and may use the Selkies API itself.
- A Rust backend is acceptable and will open the HTTP port, answer HTTP requests, and communicate with the Spicetify extension through a WebSocket. The extension is `bridge.js`; it connects to `ws://127.0.0.1:8000/ws` (Rocket `rocket_ws` route) and reconnects every 2s until the link comes back.
- `POST /debug/eval` sends its body as JavaScript to the bridge for evaluation in the Spotify renderer and returns the result as JSON (`{type:"result",id,ok,value|error}`). Gated by `DEBUG_EVAL` (default off → 403; `run-spotify.sh` sets it). 503 when no bridge is connected, 400 if evaluation threw, 504 after 10s without an answer.
- The Jellyfin catalog lives in `src/catalog.rs` (artists/albums/tracks/playlists, stable UUIDv5 ids derived from Spotify URIs). `src/spotify.rs` populates it by evaluating one collector script in the renderer via the bridge: rootlist playlists, Liked Songs (`PlaylistAPI.getPlaylist("spotify:user:me:collection")` — username part is ignored), saved albums with tracks (`Spicetify.GraphQL` `getAlbum`, field `tracksV2`), followed artists (`LibraryAPI.getContents`). A background task refreshes it at boot and every 15 min.
- Jellyfin endpoints are in `src/jellyfin/*`, mounted under both `/api` and `/`. Implemented: auth stub (static user, any credentials accepted), `/Users/{id}/Views`, the `/Users/{id}/Items` query engine (types/parent/search/pagination), item detail, playlists (read/create/add/remove), favorites, PlaybackInfo, Instant Mix (random tracks), artwork redirects to `i.scdn.co`. Audio routes stream the shared recording for every track id.
- Spotify Web API calls through CosmosAsync get 429 "Failed to fetch" from this client — use internal Platform APIs instead.

## Hard-won container lessons

- `/defaults` MUST end up owned by `abc`. `init-adduser` does `lsiown abc:abc /defaults`, but a boot race can leave it root-owned (`700 root root`). When `abc` cannot traverse `/defaults`, PulseAudio fails to start ("Failed to create secure directory (/defaults)") and every Pulse client breaks (Selkies audio, Spotify sound). The init script runs a detached watchdog loop that re-runs `lsiown -R abc:abc /defaults` until the owner is right.
- s6 kills background processes spawned from `custom-cont-init.d/*.sh` when the init phase ends, even with `&`. Long-lived helpers started there must be detached with `setsid nohup sh -c '...' >/dev/null 2>&1 &`.
- LSIO copies `/defaults/autostart` to `/config/.config/openbox/autostart` only on FIRST boot; the persistent volume keeps a stale copy forever. The init script re-syncs it (`cat /defaults/autostart > ...`) on every boot.
- The audio recorders are plain `ffmpeg -f pulse -i default` processes spawned by the init script (as root), waiting for `/defaults/native` to appear first, retrying forever if they die. Two independent processes: one `-f adts` into `/config/audio/recording.aac`, one `-f hls` (event playlist, 6s segments) into `/config/audio/hls/`. Do NOT use the ffmpeg `tee` muxer to combine them: the adts branch stays at 0 bytes while HLS writes.
- ffmpeg buffers regular-file output (~256 KB): `recording.aac` grows in bursts while HLS segments appear in real time. `/universal` streams from the file tail, so expect bursty delivery.
- Iterating on the container does NOT require a rebuild: `podman exec` to patch scripts/processes in the running container, and persist logs/state under `/config` so they survive restarts. Rebuild only once a change is validated.
- The Rust backend binary is `spotify-server` (renamed from spotify-mcp). Its build is deliberately the LAST stage of the Containerfile so app changes don't invalidate the Spotify/Spicetify layers.
- Debian trixie ships rustc 1.85; the `time` crate must be pinned (`cargo update -p time --precise 0.3.36`) or `cargo build --locked` fails inside the image.
