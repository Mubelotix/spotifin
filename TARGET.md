# Jellyfin Music API Target

This document specifies the HTTP behavior required to reimplement the part of
Jellyfin used by music clients. It is based on the current Jellyfin server
controllers and DTOs, and on actual client code from:

- `finamp-app/Finamp`
- `leinelissen/jellyfin-audio-player`
- `samyyy2311/CassetteCat`

The repositories were cloned under `/tmp/target-research/`. This is a behavioral
target, not a copy of Jellyfin's internal implementation. The server API is
larger than the music surface described here.

## Compatibility Goals

An implementation should support, in this order:

1. Authentication and the `MediaBrowser` authorization header.
2. Browsing libraries, artists, albums, tracks, genres, search, and playlists.
3. Artwork URLs and binary artwork responses.
4. Direct audio and transcoded audio playback.
5. Playback progress, favorites, played state, and resume position.
6. Instant mixes and lyrics.

Finamp is the more complete reference client. The React Native
`jellyfin-audio-player` (called Fintunes in its request headers) confirms the
minimal subset needed for a practical music player.

CassetteCat is an Android Jellyfin client that loads audio items through
`/Users/{userId}/Items` and filters the resulting library locally. It does not
use Jellyfin's `SearchTerm` or `/Search/Hints` endpoints, so its search cannot
discover tracks that have not already been returned by the initial library
enumeration.

## HTTP Conventions

### Base URL

Clients append `/api` to the configured server URL. For a server URL
`https://music.example`, the API base is:

```text
https://music.example/api
```

All paths below are relative to that base. JSON property names are PascalCase
in the wire format. Query parameter names are also PascalCase.

### Authentication Header

After login, clients send this header on authenticated requests:

```text
Authorization: MediaBrowser Client="Finamp", Device="Linux", DeviceId="device-id", Version="1.0.0", UserId="USER-ID", Token="ACCESS-TOKEN"
```

The exact client/device/version values are informational. The required values
are `Token` and usually `UserId`. Jellyfin-compatible clients may instead use
the legacy header:

```text
X-Emby-Authorization: MediaBrowser Client="Fintunes", Device="Android", DeviceId="device-id", Version="1.0.0", Token="ACCESS-TOKEN"
```

Accept both header names when reimplementing the server. For media URLs,
clients commonly put the token in `api_key` because an audio element cannot
reliably set custom headers:

```text
/Audio/{itemId}/universal?api_key=ACCESS-TOKEN&UserId=USER-ID
```

### Common Status Codes

| Status | Meaning |
|---|---|
| `200` | JSON, image, or audio response succeeded |
| `201` | Resource created, depending on endpoint |
| `204` | Successful mutation with no body |
| `302` | Redirect to remote media |
| `400` | Invalid query/body or unsupported media request |
| `401` | Missing or invalid token |
| `403` | Valid identity without permission |
| `404` | Unknown item, user, playlist, lyric, or image |
| `409` | Conflicting mutation, commonly playlist changes |

### Pagination

List endpoints return a `QueryResult` object:

```json
{
  "Items": [],
  "TotalRecordCount": 123,
  "StartIndex": 0
}
```

`StartIndex` is zero-based. `Limit` controls page size. Clients must tolerate
missing or null optional fields and should not assume `Items` is non-empty.

## Authentication API

### `POST /Users/AuthenticateByName`

Request:

```json
{
  "Username": "alice",
  "Pw": "password"
}
```

The password field is named `Pw`, not `Password`.

Response shape:

```json
{
  "User": {
    "Id": "USER-ID",
    "Name": "alice"
  },
  "AccessToken": "ACCESS-TOKEN",
  "ServerId": "SERVER-ID"
}
```

Finamp calls this directly. Fintunes uses the Jellyfin web UI to obtain the
same values from local storage, but a replacement server only needs to support
the direct endpoint.

### `GET /Users/Public`

Returns public users. Finamp defines this request but does not use it in the
observed code path. It is useful for a login user picker:

```json
[
  { "Id": "USER-ID", "Name": "alice" }
]
```

### `GET /Users/{userId}`

Returns a `UserDto`. Fintunes uses it after login as a token validation request.
At minimum return `Id`, `Name`, and the user policy/configuration fields.

### `GET /Users/Me`

Returns the authenticated `UserDto`. Not required by either analyzed music
client, but part of the user API and useful for validating a session.

### Logout

`POST /Sessions/Logout` invalidates or closes the current client session.
Finamp calls it best-effort and clears its local credentials regardless of the
response. A minimal implementation may return `204`.

## Library and Catalog API

### `GET /Users/{userId}/Views`

Returns the user's library views. Finamp uses the returned item IDs and types
to locate music libraries.

Typical response:

```json
{
  "Items": [
    {
      "Id": "LIBRARY-ID",
      "Name": "Music",
      "Type": "CollectionFolder",
      "CollectionType": "music",
      "IsFolder": true
    }
  ]
}
```

`GET /UserViews` is a non-user-scoped variant. The canonical client path is
`/Users/{userId}/Views`; implement both for compatibility.

### `GET /Users/{userId}/Items`

This is the workhorse catalog endpoint. It returns `QueryResult<BaseItemDto>`
and is used for albums, tracks, artists, genres, playlists, recent albums,
search, and folder children.

Important query parameters:

| Parameter | Use |
|---|---|
| `IncludeItemTypes` | Comma-separated types: `Audio`, `MusicAlbum`, `MusicArtist`, `MusicGenre`, `Playlist` |
| `ParentId` | Restrict results to children of an item/library |
| `Recursive` | Search descendants instead of direct children; use `true` for library-wide queries |
| `AlbumArtistIds` | Filter by album artist IDs |
| `ArtistIds` | Filter by artist IDs |
| `AlbumIds` | Filter by album IDs |
| `GenreIds` | Filter by genre IDs |
| `SearchTerm` | Case-insensitive catalog search |
| `SortBy` | Comma-separated fields such as `SortName`, `AlbumArtist`, `IndexNumber`, `ParentIndexNumber`, `DateCreated`, `SearchScore`, `Random` |
| `SortOrder` | `Ascending` or `Descending` |
| `Filters` | User filters such as favorites or played state |
| `Fields` | Comma-separated expansions, especially `MediaStreams`, `DateCreated`, `Overview`, `ChildCount`, `BasicSyncInfo`, `PrimaryImageAspectRatio`, `SortName` |
| `StartIndex` | Zero-based pagination offset |
| `Limit` | Maximum result count |
| `UserId` | User context when not already in the path |
| `EnableUserData` | Include `UserData`; clients commonly expect this for favorites/resume |
| `EnableImageTypes` | Usually `Primary,Backdrop,Banner,Thumb` |
| `ImageTypeLimit` | Usually `1` |

Observed client queries:

```text
# All tracks
/Users/{userId}/Items?IncludeItemTypes=Audio&Recursive=true&SortBy=AlbumArtist,SortName&SortOrder=Ascending&Fields=PrimaryImageAspectRatio,SortName,BasicSyncInfo,DateCreated

# All albums
/Users/{userId}/Items?IncludeItemTypes=MusicAlbum&Recursive=true&SortBy=AlbumArtist,SortName&SortOrder=Ascending&Fields=PrimaryImageAspectRatio,SortName,BasicSyncInfo,DateCreated&EnableImageTypes=Primary,Backdrop,Banner,Thumb&ImageTypeLimit=1

# Tracks in an album
/Users/{userId}/Items?ParentId={albumId}&SortBy=ParentIndexNumber,IndexNumber,SortName&Fields=MediaStreams

# Search
/Users/{userId}/Items?IncludeItemTypes=Audio,MusicAlbum,Playlist&Recursive=true&SearchTerm={term}&SortBy=SearchScore,Album,SortName&SortOrder=Ascending&Limit={limit}
```

### `GET /Items/{itemId}` and legacy user form

`GET /Users/{userId}/Items/{itemId}` is used by both clients for an album or
track detail page. `GET /Items/{itemId}` is the modern equivalent. Return a
complete `BaseItemDto`, not a reduced list result, because detail screens read
the same fields as list screens plus metadata and streams.

### Artists

`GET /Artists` returns artists. `GET /Artists/AlbumArtists` returns album
artists and is the endpoint used by both analyzed clients for the artist index.

Useful parameters are `Recursive`, `SortBy`, `SortOrder`, `SearchTerm`,
`Fields`, `EnableUserData`, `StartIndex`, `Limit`, and `UserId`.

`GET /Artists/{name}` returns an artist by its URL-encoded name. It exists in
the server API but the analyzed clients primarily use item IDs from list
results.

### Genres

`GET /Genres` returns general genres. `GET /MusicGenres` and
`GET /MusicGenres/{genreName}` are the music-specific variants. Finamp uses
`/Genres`; implement both families and return `BaseItemDto` items with `Id`,
`Name`, `Type`, `ChildCount` where available, and image fields.

### Search hints

`GET /Search/Hints?SearchTerm={term}&UserId={userId}&IncludeItemTypes=...`
returns `SearchHintResult`. It is part of the server API, but the analyzed
clients use the more general `/Users/{userId}/Items` search query instead.

## BaseItemDto Music Contract

`BaseItemDto` is a polymorphic object. The following fields are the useful
music contract; unrelated video/live-TV fields may be omitted by a music-only
implementation.

### Identity and hierarchy

| Field | Meaning |
|---|---|
| `Id` | Stable UUID string identifying the item |
| `ServerId` | Server UUID |
| `Name` | Display title |
| `SortName` | Catalog sort title |
| `Type` | `Audio`, `MusicAlbum`, `MusicArtist`, `MusicGenre`, `Playlist`, or `CollectionFolder` |
| `IsFolder` | Whether the item contains children |
| `ParentId` | Parent folder/album ID |
| `AlbumId` | Album ID for an audio track |
| `Album` | Album display name for an audio track |
| `PlaylistItemId` | Playlist entry ID when returned from a playlist |
| `IndexNumber` | Track number |
| `ParentIndexNumber` | Disc number |
| `ChildCount` | Direct child count |
| `RecursiveItemCount` | Descendant count |
| `MediaType` | Usually `Audio` for music tracks |

### Credit and metadata

| Field | Meaning |
|---|---|
| `Artists` | Artist display names |
| `ArtistItems` | Array of `{Name, Id}` artist references |
| `AlbumArtist` | Album artist display name |
| `AlbumArtists` | Array of `{Name, Id}` album artist references |
| `Genres` | Genre display names |
| `GenreItems` | Array of `{Name, Id}` genre references |
| `ProviderIds` | External IDs such as MusicBrainz IDs |
| `Overview` | Album/artist description |
| `ProductionYear` | Release year |
| `PremiereDate` | Release date |
| `DateCreated` | Import/creation timestamp |
| `DateLastMediaAdded` | Last media addition timestamp |
| `ExternalUrls` | External provider links |
| `Tags` | User/library tags |

### Images

| Field | Meaning |
|---|---|
| `ImageTags.Primary` | Cache-busting tag for primary artwork |
| `AlbumPrimaryImageTag` | Track's album artwork tag |
| `BackdropImageTags` | Backdrop artwork tags |
| `PrimaryImageItemId` | Item whose primary image should be used |
| `ImageBlurHashes` | Optional blurred placeholders |
| `PrimaryImageAspectRatio` | Layout hint |

Clients construct artwork URLs as:

```text
/Items/{itemId}/Images/Primary?tag={ImageTags.Primary}&width={width}&height={height}&format=webp
```

If `PrimaryImageItemId` is present, use it as the image item ID. A simpler
client can use the item ID and omit resizing parameters.

### Playback and file metadata

| Field | Meaning |
|---|---|
| `RunTimeTicks` | Duration in 100-nanosecond ticks |
| `Container` | Source container, for example `mp3`, `flac`, `m4a` |
| `Path` | Server-side file path; do not expose it to untrusted clients |
| `HasLyrics` | Whether lyrics are available |
| `CanDownload` | Whether download is allowed |
| `MediaSources` | Usually returned by playback info rather than catalog queries |
| `MediaStreams` | Returned when `Fields=MediaStreams`; includes codec details |
| `NormalizationGain` | Track loudness gain |
| `AlbumNormalizationGain` | Album loudness gain |
| `UserData` | Per-user play/favorite/resume state |

## User Data and Favorites

### `GET /Users/{userId}/Items/{itemId}/UserData`

Returns `UserItemDataDto`. `GET /UserItems/{itemId}/UserData` is the modern
user-context variant. Relevant fields:

```json
{
  "ItemId": "TRACK-ID",
  "Key": "TRACK-ID",
  "PlaybackPositionTicks": 123000000,
  "PlayCount": 4,
  "IsFavorite": true,
  "Likes": true,
  "Played": false,
  "LastPlayedDate": "2026-01-01T12:00:00Z",
  "Rating": 8.0,
  "UnplayedItemCount": 0
}
```

### Update user data

`POST /Users/{userId}/Items/{itemId}/UserData` accepts a `UserItemDataDto`
subset and returns the updated object. Finamp mainly uses the favorite
endpoints, but a complete implementation should accept:

- `PlaybackPositionTicks`
- `PlayCount`
- `IsFavorite`
- `Played`
- `Rating`
- `Likes` (`true` for thumbs-up, `false` for thumbs-down, omitted/null to clear)
- `LastPlayedDate`

### Favorite endpoints

Finamp uses these convenience mutations:

- `POST /Users/{userId}/FavoriteItems/{itemId}` marks an item favorite.
- `DELETE /Users/{userId}/FavoriteItems/{itemId}` removes the favorite.

Modern clients use the equivalent `/UserFavoriteItems/{itemId}` routes. In
Jellyfin, a favorite is the heart/save state (`IsFavorite`), not a dislike.
Thumbs-up and thumbs-down are the nullable `Likes` field, changed with
`POST /UserItems/{itemId}/Rating?likes=true|false` and cleared with
`DELETE /UserItems/{itemId}/Rating` (legacy user-prefixed aliases also exist).
For this Spotify-backed server, `IsFavorite` mirrors Spotify's Liked Songs;
`Likes=false` is retained as Jellyfin-local metadata because Spotify has no
corresponding library state exposed by the client bridge.

Return the updated `UserItemDataDto` if possible. The client uses
`IsFavorite`, `PlayCount`, `PlaybackPositionTicks`, and `LastPlayedDate`.

### Played state

The server also exposes:

- `POST /UserPlayedItems/{itemId}`
- `DELETE /UserPlayedItems/{itemId}`

with legacy `/Users/{userId}/PlayedItems/{itemId}` aliases. Return the updated
user data.

## Playlists

### List and read

Clients discover playlists with the normal item query:

```text
/Users/{userId}/Items?IncludeItemTypes=Playlist&Recursive=true&SortBy=SortName&SortOrder=Ascending
```

`GET /Playlists/{playlistId}/Items?UserId={userId}&SortBy=IndexNumber,SortName`
returns a `QueryResult<BaseItemDto>`. Playlist track results must contain
`PlaylistItemId`; it is not necessarily the same as the track's `Id`.

`GET /Playlists/{playlistId}` returns a `PlaylistDto`:

```json
{
  "OpenAccess": false,
  "Shares": [],
  "ItemIds": ["TRACK-ID-1", "TRACK-ID-2"]
}
```

The list result additionally commonly includes `Name`, `Id`, `ChildCount`,
`MediaType`, `CanDelete`, and `UserData`.

### Create

`POST /Playlists` accepts a `NewPlaylist` body:

```json
{
  "Name": "Favorites",
  "Ids": ["TRACK-ID-1", "TRACK-ID-2"],
  "UserId": "USER-ID",
  "MediaType": "Audio"
}
```

Return a creation result containing the new playlist `Id`.

### Add, move, remove

- `POST /Playlists/{playlistId}/Items?Ids=TRACK-ID-1,TRACK-ID-2&UserId=USER-ID`
- `POST /Playlists/{playlistId}/Items/{itemId}/Move/{newIndex}`
- `DELETE /Playlists/{playlistId}/Items?EntryIds=PLAYLIST-ENTRY-ID-1,PLAYLIST-ENTRY-ID-2`

The add endpoint uses track IDs. The delete endpoint uses playlist entry IDs.
This distinction is important for a from-scratch implementation.

`POST /Playlists/{playlistId}` updates playlist metadata. Sharing endpoints
under `/Playlists/{playlistId}/Users` are server functionality but were not
used by either analyzed music client.

## Playback and Audio Streaming

### Playback information

`GET /Items/{itemId}/PlaybackInfo?UserId={userId}` returns a
`PlaybackInfoResponse`:

```json
{
  "MediaSources": [
    {
      "Id": "SOURCE-ID",
      "Path": "/media/song.flac",
      "Protocol": "File",
      "Container": "flac",
      "Size": 12345678,
      "RunTimeTicks": 2100000000,
      "Bitrate": 1411000,
      "SupportsDirectPlay": true,
      "SupportsDirectStream": true,
      "SupportsTranscoding": true,
      "MediaStreams": [
        {
          "Type": "Audio",
          "Index": 0,
          "Codec": "flac",
          "Channels": 2,
          "SampleRate": 44100,
          "BitRate": 1411000,
          "IsDefault": true
        }
      ]
    }
  ],
  "PlaySessionId": "PLAY-SESSION-ID",
  "ErrorCode": null
}
```

Finamp reads `MediaSources`, `PlaySessionId`, and `ErrorCode`. The relevant
`MediaSourceInfo` fields are:

- `Id`, `Path`, `Protocol`, `Type`, `Container`, `Size`, `Name`
- `RunTimeTicks`, `Bitrate`, `IsRemote`
- `SupportsDirectPlay`, `SupportsDirectStream`, `SupportsTranscoding`
- `MediaStreams`, `DefaultAudioStreamIndex`
- `TranscodingUrl`, `TranscodingContainer`, `TranscodingSubProtocol`

The server also accepts:

```text
POST /Items/{itemId}/PlaybackInfo?UserId={userId}
```

The POST body is `PlaybackInfoDto`, which carries a client device profile and
requested media constraints. It is the extensible form for clients that need
to tell the server exactly which containers, codecs, bitrate, channels,
transcoding protocols, and direct-play capabilities they support. A compatible
implementation should accept an empty body as well as a populated profile and
return the same `PlaybackInfoResponse` shape as the GET endpoint.

### Direct audio

`GET /Audio/{itemId}/stream` returns the source audio or a server-selected
transcode. It accepts a large device-profile query surface. The parameters
needed by a music-only implementation are:

| Parameter | Meaning |
|---|---|
| `container` | Desired output container |
| `mediaSourceId` | Selected `MediaSourceInfo.Id` |
| `deviceId` | Client device ID |
| `audioCodec` | Requested codec |
| `audioBitRate` | Target bitrate |
| `audioChannels` / `maxAudioChannels` | Channel constraints |
| `audioSampleRate` | Sample-rate constraint |
| `maxAudioBitDepth` | Bit-depth constraint |
| `startTimeTicks` | Resume/seek start offset |
| `playSessionId` | Playback session ID |
| `audioStreamIndex` | Selected audio stream |
| `static` | Request a static file when possible |
| `tag` | Cache/version tag |
| `transcodingContainer` | Transcoding output container |
| `transcodingProtocol` | `http` or `hls` style protocol |
| `enableAudioVbrEncoding` | Enable VBR encoding, default true |

The route has a container extension alias:

```text
GET /Audio/{itemId}/stream.{container}
```

Support `HEAD` on both routes. `Range` requests are important for seeking and
must return `206 Partial Content`, `Content-Range`, `Accept-Ranges: bytes`,
and a correct `Content-Length` when direct-playing.

### Universal audio

`GET /Audio/{itemId}/universal` is the preferred route in Fintunes. It selects
direct play or transcoding from a compact device request.

Important query parameters:

| Parameter | Meaning |
|---|---|
| `container` | Comma-separated acceptable containers, for example `mp3,aac,flac,wav,ogg` |
| `mediaSourceId` | Selected media source |
| `deviceId` | Device identity |
| `UserId` | User identity |
| `audioCodec` | Requested codec, often `aac` |
| `transcodingContainer` | Transcode output, often `aac` |
| `transcodingProtocol` | Usually `http` |
| `audioBitRate` | Target bitrate, often `320000` |
| `maxAudioChannels` | Maximum channels |
| `maxAudioSampleRate` | Maximum sample rate |
| `maxAudioBitDepth` | Maximum bit depth |
| `startTimeTicks` | Seek/resume offset |
| `enableRemoteMedia` | Permit remote source media |
| `enableAudioVbrEncoding` | VBR preference |
| `enableRedirection` | Permit `302` remote redirects |
| `api_key` | Token for audio-element requests |

Fintunes checks `Content-Type` and `Content-Length` to distinguish direct play
from transcoded playback. Return an audio MIME type such as
`audio/flac`, `audio/mpeg`, `audio/aac`, or `audio/mp4`.

### HLS

The server exposes audio HLS routes for clients that require segmented media:

- `GET /Audio/{itemId}/master.m3u8`
- `GET /Audio/{itemId}/main.m3u8`
- `GET /Audio/{itemId}/hls1/{playlistId}/{segmentId}.{container}`

Legacy segment routes include `/Audio/{itemId}/hls/{segmentId}/stream.mp3` and
`stream.aac`. A minimal music player can use universal HTTP audio instead, but
an implementation claiming Jellyfin streaming compatibility should provide HLS
playlists, segment URLs, byte/range correctness, and stable playlist IDs.

## Playback Reporting

Clients report playback to keep server state, resume positions, and “recently
played” views correct.

### Start

`POST /Sessions/Playing` accepts `PlaybackStartInfo`, including:

- `ItemId`
- `MediaSourceId`
- `PlaySessionId`
- `PositionTicks`
- `PlayMethod`: `DirectPlay`, `DirectStream`, or `Transcode`
- `AudioStreamIndex`
- `CanSeek`
- `RepeatMode` and `ShuffleMode`
- `VolumeLevel`, `IsMuted`, `IsPaused`
- `DeviceProfile`

### Progress

`POST /Sessions/Playing/Progress` accepts `PlaybackProgressInfo`:

```json
{
  "ItemId": "TRACK-ID",
  "MediaSourceId": "SOURCE-ID",
  "PlaySessionId": "PLAY-SESSION-ID",
  "PositionTicks": 123000000,
  "IsPaused": false,
  "IsMuted": false,
  "VolumeLevel": 80,
  "PlayMethod": "DirectPlay",
  "CanSeek": true,
  "RepeatMode": "RepeatNone",
  "ShuffleMode": "Sorted",
  "PlaybackRate": 1
}
```

Finamp sends this periodically. Fintunes sends progress with
`PlaySessionId` and `MediaSourceId` often set to the track ID or a client
constant. Accept that behavior.

### Stop

`POST /Sessions/Playing/Stopped` accepts the same identifying fields and final
`PositionTicks`. Persist the final position and update played state according
to the server's configured play threshold.

`POST /Sessions/Playing/Ping?PlaySessionId={id}` keeps a session alive and is
useful for long tracks, though neither analyzed client depends on it.

## Instant Mix

The server has specialized routes:

- `GET /Songs/{itemId}/InstantMix`
- `GET /Albums/{itemId}/InstantMix`
- `GET /Playlists/{itemId}/InstantMix`
- `GET /MusicGenres/{name}/InstantMix`
- `GET /Artists/{itemId}/InstantMix`
- `GET /Artists/InstantMix?Id={id}`
- `GET /MusicGenres/InstantMix?Id={id}`
- `GET /Items/{itemId}/InstantMix`

Common parameters are `UserId`, `Limit`, and sometimes an ID/name list. Return
a `QueryResult<BaseItemDto>` containing playable `Audio` items. Finamp uses
`/Items/{id}/InstantMix`; Fintunes uses the same route for track mixes.

## Lyrics

### `GET /Audio/{itemId}/Lyrics`

Returns `LyricDto` or `404`:

```json
{
  "Metadata": {
    "Artist": "Artist",
    "Album": "Album",
    "Title": "Track",
    "Author": "Provider",
    "Length": 240,
    "By": "provider",
    "Offset": 0,
    "Creator": "provider",
    "Version": "1",
    "IsSynced": true
  },
  "Lyrics": [
    { "Text": "First line", "Start": "00:00:10.000" }
  ]
}
```

Clients consume `Metadata`, `Lyrics[].Text`, and `Lyrics[].Start`. `Start` may
be represented as a time span by generated clients; normalize it to
milliseconds internally.

The server also exposes management/provider routes that are not needed for
normal playback:

- `POST /Audio/{itemId}/Lyrics` uploads external plain-text lyrics with a
  required `fileName` query parameter.
- `DELETE /Audio/{itemId}/Lyrics` removes lyrics.
- `GET /Audio/{itemId}/RemoteSearch/Lyrics` searches providers.
- `POST /Audio/{itemId}/RemoteSearch/Lyrics/{lyricId}` downloads provider lyrics.
- `GET /Providers/Lyrics/{lyricId}` gets provider lyrics.

## Artwork

Both clients use item artwork heavily. The most useful routes are:

- `GET /Items/{itemId}/Images/Primary`
- `GET /Items/{itemId}/Images/{imageType}`
- `GET /Items/{itemId}/Images/{imageType}/{index}`
- `GET /Artists/{name}/Images/{imageType}/{index}`
- `GET /MusicGenres/{name}/Images/{imageType}`
- `GET /Genres/{name}/Images/{imageType}`

Support query parameters such as `tag`, `format`, `maxWidth`, `maxHeight`,
`quality`, `percentPlayed`, and `unplayedCount`. Return the binary image with a
correct `Content-Type`, cache headers, and `404` for missing artwork.

Image metadata is available from `GET /Items/{itemId}/Images` and includes
image type, index, width, height, size, blur hash, and tag information.

## Client Usage Matrix

| Capability | Finamp | Fintunes | Recommendation |
|---|---:|---:|---|
| `POST /Users/AuthenticateByName` | Yes | WebView-derived credentials | Must implement |
| `GET /Users/{id}/Views` | Yes | No | Implement |
| `GET /Users/{id}/Items` | Yes, heavily | Yes, heavily | Core endpoint |
| `GET /Users/{id}/Items/{item}` | Yes | Yes | Implement |
| `GET /Artists/AlbumArtists` | Yes | Yes | Implement |
| `GET /Genres` | Yes | No | Implement |
| `GET /Items/{id}/InstantMix` | Yes | Yes | Implement |
| `GET /Items/{id}/PlaybackInfo` | Yes | No | Implement |
| `GET /Audio/{id}/universal` | No | Yes | Implement |
| `GET /Audio/{id}/stream` | Indirectly via playback info | No | Implement |
| `GET /Audio/{id}/Lyrics` | No | Yes | Implement |
| Primary artwork endpoint | Defined but not used | Yes | Implement |
| Favorite endpoints | Yes | No | Implement |
| Playlist create/add/remove | Yes | No | Implement |
| Playlist item listing | Yes | Yes | Implement |
| Playback start/progress/stop | Yes | Progress | Implement |
| `GET /Users/Public` | Defined, unused | No | Optional |
| Search hints | No | No | Optional |
| Remote lyric management | No | No | Optional |
| HLS audio | No direct use observed | No | Compatibility extension |
| Session control commands | No | No | Do not prioritize |
| Library administration | No | No | Do not implement for a music client |

## What Is Actually Useful

### High value

- A single flexible `/Users/{userId}/Items` query endpoint.
- Stable UUID item IDs and explicit `Type` values.
- `Fields=MediaStreams` for codec/sample-rate/bit-depth selection.
- `UserData` embedded in list/detail results.
- Artwork tags and `PrimaryImageItemId`.
- Range-capable audio responses.
- Progress reporting with 100 ns ticks.
- Playlist entry IDs distinct from track IDs.

### Low value for a music-only reimplementation

These exist in the server but were not used by either analyzed app's normal
music flows:

- Public-user listing when direct username authentication is available.
- Search hints; item search was sufficient in both clients.
- Library administration and physical path APIs.
- Theme songs/videos and video-specific HLS routes.
- Session remote-control commands, viewing commands, and sync play.
- Remote lyric search/download unless lyric editing is a product requirement.
- Playlist sharing and permission management.
- Image upload/delete/reordering.

Do not remove fields from `BaseItemDto` merely because they are unused by one
screen. Clients request broad `Fields` values and may decode optional fields
without immediately reading them.

## Reimplementation Model

The minimum useful data model is:

```text
User(id, name, password_hash, policy, configuration)
Library(id, name, collection_type=music)
Artist(id, name, sort_name, overview, provider_ids, artwork)
Album(id, name, sort_name, album_artist_ids, year, overview, artwork)
Track(id, album_id, disc_number, track_number, title, artists, genres,
      path, container, duration_ticks, codec, channels, sample_rate,
      bit_rate, bit_depth, lyrics, artwork, provider_ids)
Playlist(id, owner_user_id, name, item_entries)
PlaylistEntry(id, playlist_id, track_id, index)
UserItemData(user_id, item_id, favorite, played, play_count,
             playback_position_ticks, rating, last_played_date)
PlaybackSession(id, user_id, device_id, item_id, source_id, position_ticks,
                play_method, last_seen)
```

Use UUID strings on the wire. Store duration and positions as integer ticks;
one second is `10_000_000` ticks. Keep source media metadata separate from
catalog metadata so one track can expose direct play and transcode options.

### Request processing order

1. Parse and validate the `MediaBrowser` token.
2. Resolve `UserId` from the path/query/header and enforce access.
3. Resolve item IDs and user-specific visibility.
4. Apply type, parent, artist, album, genre, search, and recursive filters.
5. Sort deterministically.
6. Apply pagination.
7. Project requested `Fields` and attach `UserData` when enabled.
8. Serialize PascalCase JSON.

### Security requirements

- Never return server filesystem paths to an untrusted client unless matching
  Jellyfin's trusted-client behavior is explicitly required.
- Treat `api_key` as equivalent to the authorization token.
- Validate item IDs, playlist entry IDs, image types, containers, and stream
  options.
- Do not let a user read another user's favorites, resume state, or private
  playlist entries.
- Restrict lyric upload, image mutations, library mutations, and playlist
  sharing to their respective permissions.

## Reference Client Details

### Finamp

Finamp uses a generated Chopper client plus a hand-written service layer. It
actually calls approximately twenty endpoints. Its catalog abstraction is
deliberately broad: albums, tracks, artists, genres, search, and instant mixes
all become variations of `/Users/{userId}/Items`.

It stores `AccessToken`, user ID, server ID, server URL, and device identity.
There is no token refresh flow. Logout clears local credentials after a
best-effort `POST /Sessions/Logout`.

### jellyfin-audio-player / Fintunes

Fintunes uses a small `fetchApi` wrapper. It injects the MediaBrowser header,
maps `401`/`403` to authentication failure and `404` to missing resources, and
uses `api_key` on audio URLs. It reads a narrower but representative DTO set:
artist/album/track identity, image tags, streams, user data, lyrics, and
playlist entry metadata.

Its login flow extracts Jellyfin credentials from the web UI's local storage
instead of calling `AuthenticateByName`; this is a client choice, not a server
requirement.

## Minimal Acceptance Tests

An implementation is compatible with the analyzed clients when these tests
pass:

1. Authenticate a user and make an authenticated catalog request.
2. List a music view, then list albums recursively.
3. Open an album and list tracks sorted by disc and track number.
4. Return primary artwork using an image tag.
5. Return playback info with at least one source and audio stream.
6. Serve a direct audio request with `Content-Length` and byte ranges.
7. Serve a universal audio request with a valid audio content type.
8. Report start, progress, and stop, then return updated `UserData`.
9. Mark a track favorite and remove the favorite.
10. Create a playlist, add tracks, list entries, and remove an entry by its
    playlist entry ID.
11. Return an instant mix containing playable tracks.
12. Return synced and unsynced lyrics with the documented shape.
13. Restart the client without changing IDs or losing user state.
