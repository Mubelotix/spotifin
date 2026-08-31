# Spotify MCP Client Compatibility

| Feature | [Finamp](https://github.com/finamp-app/finamp) | [Fintunes](https://github.com/leinelissen/jellyfin-audio-player) | [Yuzic](https://github.com/eftpmc/yuzic) | [Jellify](https://github.com/Jellify-Music/App) | [CassetteCat](https://github.com/samyyy2311/CassetteCat) | [Wavio](https://github.com/Joel-Mercier/wavio) | [Linthra](https://github.com/TheZupZup/Linthra) | [Coppelia](https://github.com/j6k4m8/coppelia) | [JellyBoxPlayer](https://github.com/avdept/JellyBoxPlayer) |
|---|---|---|---|---|---|---|---|---|---|
| Login | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| View library | ✅ | ✅ | ✅ | ✅ | ⚠️ Without Playlists | ✅ | ✅ | ✅ | ✅ |
| View artist | ✅ | ⚠️ Saved content only | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ✅ |
| View album | ✅ | ⚠️ From search | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ⚠️ Saved content only | ✅ | ✅ |
| Remote Search | ✖️ | ✅ | ⚠️ Only songs | ⚠️ Only songs | ✖️ | ⚠️ Only songs | ✖️ | ⚠️ Only songs | ✖️ |
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

Feel free to report your experience with apps by opening new issues. Most issues may be solved by improving the behavior of our backend.

## Spotify Autoplay

Autoplay is implemented under Jellyfin's **Instant Mix** feature.
This corresponds to Spotify's recommended songs and fills the queue with roughly two and a half hours of Spotify recommendations.

Spotify's **Daily Mixes** and **Radios**, normally shown on the Spotify home page, are exposed as playlists.

## For Client Devs

Making your app run better with Spotify is easy! Only a few tweaks to your existing implementations are necessary to make them flawless:

1. Always send `Sessions/Playing` with the selected `ItemId` to help the backend know what it should prioritize
2. Start playing audio directly when you receive data, don't wait to gather a 20-second buffer, as it will take 20 seconds
3. Downloads and preloads may return 409 errors. They are safe to retry.
4. Use stable Jellyfin `Id` values, not titles or other metadata.

It's easy to look at what's going on spotify's end, just open http://localhost:8030 to watch a live screen capture.
