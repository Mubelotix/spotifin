# spotify-mcp

A Jellyfin-compatible music server backed by a **real Spotify desktop client** running in a rootless podman container (LinuxServer Selkies baseimage, session streamed to the browser). The Rust backend speaks the Jellyfin API; the audio it serves is whatever Spotify actually plays, captured from PulseAudio by ffmpeg.

## Goal

Any Jellyfin music client (Finamp, Fintunes) can point at this server, browse the Spotify account like a local library (playlists, Liked Songs, saved albums, followed artists), search **all of Spotify**, and play tracks with correct lengths, seeking, favorites, playlists and lyrics.

Core idea: the backend never talks to spotify.com. It drives the logged-in client from inside its own renderer (via a WebSocket to the `bridge.js` Spicetify extension) using the same internal APIs the UI uses — indistinguishable from a human clicking around.

## Resources

- `TARGET.md` — behavioral spec of the Jellyfin API subset (derived from Jellyfin server, Finamp, jellyfin-audio-player sources).
- `spicetify-dev-docs.md` — vendored spicetify development docs (Platform, CosmosAsync, GraphQL), one greppable file, sections delimited by `=== <path> ===`. Docs drift: verify against the live client before relying on anything.
- `bridge.js` — Spicetify extension; connects to `ws://127.0.0.1:8000/ws`, reconnects every 2 s, evaluates JS sent by the backend (`POST /debug/eval`, gated by `DEBUG_EVAL`) and returns results as JSON.
- `src/catalog.rs` — in-memory model; stable UUIDv5 item IDs derived from Spotify URIs.
- `src/spotify.rs` — collector (builds the catalog by evaluating scripts in the renderer) plus live helpers: search, playlist mutations, lyrics.
- `src/player.rs` — on-demand playback, recorder resets, idle watchdog.
- `src/jellyfin/` — HTTP endpoints, mounted under both `/api` and `/`.
- `Containerfile` / `run-spotify.sh` — everything container-related lives in the Containerfile (inline config with `RUN printf '%s\n' ...`; podman 4.9 has no heredocs). Keep the file count minimal.

## Usage

```bash
podman build -t spicetify-web .
./run-spotify.sh   # then open https://localhost:8031 (user: spotify, password: spotify)
```

Spicetify plugins are env vars on `run-spotify.sh`: `SPICETIFY_EXTENSIONS` (default `adblock.js bridge.js`), `SPICETIFY_CUSTOM_APPS`, `SPICETIFY_THEME`.

## How it works

- **Catalog**: populated by one collector script evaluated in the renderer (rootlist playlists, Liked Songs via `PlaylistAPI.getPlaylist("spotify:user:me:collection")`, saved albums via `GraphQL getAlbum`/`tracksV2`, followed artists via `LibraryAPI.getContents`). Refreshed at boot and every 15 min; refreshes **merge**, never replace (search-ingested tracks and user data survive).
- **Search**: `SearchTerm` queries also hit Spotify search in the renderer; results are ingested with stable IDs, so any track on Spotify becomes playable by ID.
- **Playback**: an audio request for a known item navigates to `/track/{id}`, presses play, verifies via `PlayerAPI.getState`, then resets the recorder. Responses end after exactly the track's expected byte count (recorder runs 192 kbps CBR). Byte-range requests and HEAD supported for seek bars.
- **Idle watchdog**: after `PLAYBACK_IDLE_TIMEOUT_SECS` (default 600) without requests, pause + drain the queue — pausing alone loses the race against autoplay advancing between tracks. Never fires mid-capture, so pauses land at song boundaries.
- **User data**: favorites stored server-side; playback reports update counts/position/last-played (informational only — the Spotify client owns the real cursor).
- **Playlists**: create/add/remove/reorder are real client operations, synced both ways.
- Auth is a stub: one static user, any credentials accepted. Lyrics come from the client's color-lyrics endpoint.

## Hard-won API lessons (verified live)

- `api.spotify.com` is blocked from this client ("Failed to fetch"); the internal Platform APIs and `spclient.wg.spotify.com` work fine via CosmosAsync.
- Rocket query/form guards match field names case-sensitively: always declare `#[field(name = "PascalCase")]`.
- Playlist ops: create via `RootlistAPI.applyModification({operation:"create",createItemKind:1,name})`; add/remove/move via `PlaylistAPI.add/remove/move(uri, rows, {})` — the `{}` options argument is mandatory. Move anchors accept `"start"`, `"end"` or row objects with `uid`; a bare URI string silently degrades to move-to-top.
- Never hold a catalog lock across a bridge await; re-read playlists after mutations instead of predicting state.

## Container lessons

- Custom init scripts go to `/custom-cont-init.d/*.sh`, app autostart to `/defaults/autostart`.
- Root-owned leftovers under `/config` or `/defaults` break everything (black screen, PulseAudio failure). The init script chowns both to `abc` at boot, with a detached loop that keeps re-fixing `/defaults`.
- Stale `/config/.cache/spotify/Singleton*` and `pending/` locks crash Spotify silently at startup — cleaned every boot. Spotify also needs `dbus-launch` or it busy-spins at 100 % CPU.
- s6 kills background processes started from init scripts when the phase ends, even with `&` — detach long-lived helpers with `setsid nohup sh -c '...' >/dev/null 2>&1 &`.
- Two independent recorders: `-f adts` into `recording.aac` (what we stream) and `-f hls` into `/config/audio/hls/`. Never combine them with ffmpeg's `tee` muxer (adts stays empty). ffmpeg buffers file output ~256 KB → bursty delivery. Recorder resets kill only the ffmpeg writer (anchored pkill — matching the supervisor shell kills the respawn loop).
- Iterate without rebuilding: `podman exec` into the running container, persist state under `/config`. Rebuild only once validated. The Rust build is deliberately the LAST Containerfile stage.
- Debian trixie ships rustc 1.85; pin `time` (`cargo update -p time --precise 0.3.36`) or `--locked` builds fail inside the image.
