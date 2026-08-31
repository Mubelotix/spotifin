# Spotify MCP Client Compatibility

| Feature | Finamp | Fintunes | Yuzic | Jellify | CassetteCat | Wavio | Linthra | Coppelia | JellyBoxPlayer |
|---|---|---|---|---|---|---|---|---|---|
| Login | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| View library | ✅ | ✅ | ✅ | ✅ | ⚠️ Without Playlists | ✅ | ✅ | ✅ | ✅ |
| View artist | ✅ | ⚠️ Saved content only | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ✅ |
| View album | ✅ | ⚠️ From search | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ✅ |
| Remote Search | ⚠️ Only songs | ✅ | ⚠️ Only songs | ⚠️ Only songs | ✖️ | ⚠️ Only songs | ✖️ | ⚠️ Only songs | ✖️ |
| Play an uncached track | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ Slow | ⚠️ Unreliable | ✅ |
| Play a cached track | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Download tracks | ✅ | ✅ Whole albums | ✅ Whole album or artist | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Autoplay | ✅ | ⚠️ From search | ✅ | ✅ | ✅ | ✅ | ✖️ | ✖️ | ✖️ |
| Seek forward and backward |  |  | | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Like/Unlike | ✅ | ✖️ | ✅ | ✅ | ✅ | ✅ | ✖️ | ✅ | ⚠️ Only in lists |
| Create a playlist | ✅ | ✖️ | ✅ | ✅ | ✖️ Local only | ✅ | ✅ | ✅ | ✅ |
| Edit playlist content | ✅ | ✖️ | ✅ | ✅ | ✖️ Local only | ✅ | ✅ | ✅ | ✅ |
| Remove playlist | ✖️ | ✖️ | ✅ | ✅ | ✖️ Local only | ✅ | ✅ | ✅ | ✅ |
| Lyrics | ✖️ | ✅ | ✅ | ✅ | ⚠️ From LRCLib | ✅ | ✅ | ✖️ | ✅ |
| Follow artist | ✅ | ✅ | ✖️ | ✅ | ✖️ | ✅ | ✖️ | ✅ | ✖️ |
| Save album | ✅ | ✖️ | ✖️ | ✅ | ✖️ | ✅ | ✖️ | ✅ | ✅ |

### Legend

✅ Verified  
⚠️ Partially working  
✖️ Not implemented in the app  
❌ Broken

## Spotify Recommendations

Clients that support autoplay use Jellyfin's **Instant Mix** feature. This
corresponds to Spotify's recommended songs and fills the queue with roughly two
and a half hours of Spotify recommendations.

Spotify's **Daily Mixes** and **Radios**, normally shown on the Spotify home
page, are exposed as playlists.

## For Client Authors

The server has one shared Spotify player and recorder. For reliable playback:

- Use stable Jellyfin `Id` values, not titles or other metadata.
- Use the media source returned by `PlaybackInfo`.
- Open only the selected track's uncached stream; do not preload or probe queued tracks.
- Send `Sessions/Playing` with the selected `ItemId`, then send progress and stopped reports normally.

Opening multiple uncached tracks concurrently can switch Spotify repeatedly or produce `409 Conflict`.
