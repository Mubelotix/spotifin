<h1 align="center">Spotifin</h1>

<p align="center">
  <strong>Spotify's catalog. Jellyfin's API.</strong><br>
</p>

<p align="center">
  <a href="https://spicetify.app/">Spicetify</a> for ad blocking
  &nbsp;&bull;&nbsp;
  <a href="https://github.com/linuxserver/docker-baseimage-selkies">Selkies</a> for the desktop
  &nbsp;&bull;&nbsp;
  <a href="https://jellyfin.org/">Jellyfin</a> for the clients you already love
</p>

This software runs the official Spotify desktop client inside a container and hijacks it, so that it can expose the account's library through a Jellyfin-compatible server.

Use existing open-source apps, or build new experiences on top of the open API.

> [!WARNING]
> Spotifin is an independent, unofficial project.
> It is not affiliated with, endorsed by, sponsored by, or otherwise associated with Spotify AB or its subsidiaries.

## What You Get

- **Ad blocking by default:** Spicetify's `adblock.js` extension blocks Spotify advertisements so playback stays uninterrupted.
- **Your Spotify library:** Playlists, Liked Songs, saved albums, followed artists, artwork, and metadata appear as a Jellyfin music library.
- **Search across Spotify:** Some clients can search across Spotify; other clients may limit search to the catalog they have already loaded.
- **Real playback:** A requested track is played by the real Spotify desktop client and its PulseAudio output is captured and served as AAC.
- **Fast repeats:** Completed captures are cached locally, so replaying a track does not need to switch Spotify again.
- **Playlists and favorites:** Jellyfin playlist operations, favorites, playback progress, and resume data are exposed through the server.
- **Lyrics:** Lyrics are read from Spotify's lyrics service and exposed through the Jellyfin endpoints.
- **Autoplay:** Jellyfin's Instant Mix maps to Spotify recommendations and can fill the queue for roughly two and a half hours.
- **Daily Mixes and Radios:** Spotify's home-page mixes and radios are surfaced as playlists.

## Quick Start

### Open the services

| Address | Purpose |
|---|---|
| `https://localhost:8031` | Selkies streamed Spotify desktop, recommended |
| `http://localhost:8030` | Alternate browser endpoint |
| `http://localhost:8032` | Jellyfin server URL for compatible clients |

1. Open `https://localhost:8031` and sign in to the desktop with `spotify` / `spotify`.
2. Sign in to Spotify inside the streamed desktop session.
3. Wait for the library collector to finish its first pass.
4. Configure your Jellyfin client with `http://<host>:8032`. If it asks for credentials, the default pair is `spotify` / `spotify`.

The Selkies endpoint may use a locally generated certificate, so a browser
warning on the first visit is expected. If a client cannot connect, use port
`8032`, not the browser desktop port.

> [!WARNING]
> The included authentication is intentionally minimal and the launcher
> publishes the Jellyfin API on port `8032`. Keep it on a trusted network or
> place it behind your own firewall, reverse proxy, and authentication layer
> before exposing it to the internet.

> [!NOTE]
> The image does not contain the Spotify client itself. On first boot the
> container downloads it (~130 MB) from Spotify's official apt repository and
> stores it in `/config/spotify-client`, where it persists across restarts and
> container recreations. Delete that folder to force a fresh download.

## Cache Structure

We store data as files, so it's easy to process for the few of you that need it.

```text
/config/
├── recording.aac
├── spotify-client/
│   └── usr/…            # Spotify desktop client, downloaded on first boot
├── cache/
│   ├── <id>.aac
│   ├── playlist-<id>.json
│   ├── track-<id>.json
│   ├── album-<id>.json
│   ├── artist-<id>.json
│   └── lyrics-<id>.json
└── hls/
    ├── main.m3u8
    └── segment-*.ts
```

- `recording.aac` is the shared live capture. It is continuously replaced as
  Spotify switches tracks and should not be treated as a completed recording.
- `<id>.aac` contains completed track captures. The first request
  for an uncached track starts a live capture; later requests can be served
  directly from this file.
- `playlist-*.json` stores playlist snapshots so the catalog can come back
  quickly while Spotify is still starting.
- `track-*.json` stores tracks discovered through remote search, allowing them
  to remain playable after a restart.
- `album-*.json` and `artist-*.json` store album track lists and artist
  discographies fetched when those pages are opened.
- `lyrics-*.json` stores lyrics fetched for a track's first lyrics request.
- `hls/` contains the live HLS playlist and segments used by clients that
  request HLS playback. It is operational state, not the permanent audio cache.

The IDs in filenames are stable UUIDs derived from Spotify URIs, not readable
track titles. Temporary `.tmp` files may briefly appear while JSON cache files
are being written safely.

## Client Support

The server speaks the part of the Jellyfin API needed by the following music clients. This list is not exhaustive.

Client capabilities differ because each app implements a different portion of the Jellyfin music experience. The verified snapshot below makes those differences explicit.

| Feature | [Finamp](https://github.com/finamp-app/finamp) | [Fintunes](https://github.com/leinelissen/jellyfin-audio-player) | [Yuzic](https://github.com/eftpmc/yuzic) | [Jellify](https://github.com/Jellify-Music/App) | [CassetteCat](https://github.com/samyyy2311/CassetteCat) | [Wavio](https://github.com/Joel-Mercier/wavio) | [Linthra](https://github.com/TheZupZup/Linthra) | [Coppelia](https://github.com/j6k4m8/coppelia) | [JellyBoxPlayer](https://github.com/avdept/JellyBoxPlayer) |
|---|---|---|---|---|---|---|---|---|---|
| Login | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| View library | ✅ | ✅ | ✅ | ✅ | ⚠️ Without Playlists | ✅ | ✅ | ✅ | ✅ |
| View artist | ✅ | ⚠️ Saved content only | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ✅ |
| View album | ✅ | ⚠️ From search | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ✅ |
| Remote search | ✖️ | ✅ | ⚠️ Only songs | ⚠️ Only songs | ✖️ | ⚠️ Only songs | ✖️ | ⚠️ Only songs | ✖️ |
| Play an uncached track | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ Slow | ⚠️ Unreliable | ✅ |
| Play a cached track | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Download tracks | ✅ | ✅ Whole albums | ✅ Whole album or artist | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Autoplay | ✅ | ⚠️ From search | ✅ | ✅ | ✅ | ✅ | ✖️ | ✖️ | ✖️ |
| Seek forward and backward | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Like/Unlike | ✅ | ✖️ | ✅ | ✅ | ✅ | ✅ | ✖️ | ✅ | ⚠️ Only in lists |
| Create a playlist | ✅ | ✖️ | ✅ | ✅ | ✖️ Local only | ✅ | ✅ | ✅ | ✅ |
| Edit playlist content | ✅ | ✖️ | ✅ | ✅ | ✖️ Local only | ✅ | ✅ | ✅ | ✅ |
| Remove playlist | ✖️ | ✖️ | ✅ | ✅ | ✖️ Local only | ✅ | ✅ | ✅ | ✅ |
| Lyrics | ✖️ | ✅ | ✅ | ✅ | ⚠️ From LRCLib | ✅ | ✅ | ✖️ | ✅ |
| Follow artist | ✅ | ✅ | ✖️ | ✅ | ✖️ | ✅ | ✖️ | ✅ | ✖️ |
| Save album | ✅ | ✖️ | ✖️ | ✅ | ✖️ | ✅ | ✖️ | ✅ | ✅ |

| ✅ | ⚠️ | ✖️ | ❌ |
|---|---|---|---|
| Verified | Partially working | Not implemented in the app | Broken |

> Please tell me about your experience with apps by opening issues!

## Trade-offs

- The first play of a track is a live capture, not an instant file download. It gets cached for future streams.
- Audio comes is recorded from Spotify's desktop output, so it's not the same file Spotify has on their servers.
- This project is designed for personal, local use rather than as a public multi-user streaming service.

## For Client Developers

Making a Jellyfin music client work smoothly with Spotifin requires very little changes:

1. Always send `Sessions/Playing` before you start streaming so the server knows which track to prioritize.
2. Start playing audio as soon as data arrives. Do not wait to gather a 20-second buffer, because the capture itself will take 20 seconds.
3. Downloads and preloads may return `409` errors. They are safe to retry.
4. Use stable Jellyfin `Id` values, not titles or other metadata.

## Development

Once the container is running, the backend can be rebuilt and reinjected into the container without having to rebuild it entirely:

```bash
./dev-backend.sh
```

## Advanced Spicetify Configuration

The default extension set is:

```text
adblock.js bridge.js
```

Customize the embedded Spotify installation when launching the container:

```bash
SPICETIFY_EXTENSIONS="adblock.js bridge.js" \
SPICETIFY_CUSTOM_APPS="lyrics-plus" \
SPICETIFY_THEME="YourTheme" \
./run-spotify.sh
```

The variables are space-separated lists where applicable:

| Variable | Default | Purpose |
|---|---|---|
| `SPICETIFY_EXTENSIONS` | `adblock.js bridge.js` | Extensions applied to Spotify. Keep `bridge.js` enabled for Jellyfin control; removing `adblock.js` allows advertisements to return. |
| `SPICETIFY_CUSTOM_APPS` | empty | Spicetify custom apps to enable. |
| `SPICETIFY_THEME` | empty | Spicetify theme to apply. |
