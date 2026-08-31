# Spotify MCP Client Compatibility

| Feature | Finamp | Fintunes | Yuzic | CassetteCat | Wavio | Linthra | Coppelia | JellyBoxPlayer |
|---|---|---|---|---|---|---|---|---|
| Login | ✅ |  |  |  |  |  |  |  |
| View library | ✅ |  |  |  |  |  |  |  |
| View artist | ✅ |  |  |  |  |  |  |  |
| View album | ✅ |  |  |  |  |  |  |  |
| Search | ⚠️ Only songs |  |  |  |  |  |  |  |
| Play an uncached track | ✅ |  |  |  |  |  |  |  |
| Play a cached track | ✅ |  |  |  |  |  |  |  |
| Download tracks |  |  |  |  |  |  |  |  |
| Autoplay | ✅ |  |  |  |  |  |  |  |
| Skip tracks | ✅ |  |  |  |  |  |  |  |
| Seek forward and backward | ✅ |  |  |  |  |  |  |  |
| Like/Unlike | ✅ |  |  |  |  |  |  |  |
| Create a playlist | ✅ |  |  |  |  |  |  |  |
| Edit playlist content | ✅ |  |  |  |  |  |  |  |
| Remove playlist | ✖️ |  |  |  |  |  |  |  |
| Display lyrics | ✖️ |  |  |  |  |  |  |  |
| Follow artist | ✅ |  |  |  |  |  |  |  |
| Save album | ✅ |  |  |  |  |  |  |  |

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
