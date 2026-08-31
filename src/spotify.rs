use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

use crate::bridge::{eval_on_bridge, BridgeState};
use crate::catalog::{self, Album, Artist, Catalog, Playlist, PlaylistEntry, Track};

/// One eval per refresh: pull playlists (rootlist + liked songs), then saved
/// albums with their tracks and followed artists. Runs inside the Spotify
/// renderer, so it is the client querying its own backend.
const COLLECT_JS: &str = r#"
(async () => {
    async function dumpPlaylist(uri, fallbackName, liked = false) {
        const pageSize = 200;
        const first = await Spicetify.Platform.PlaylistAPI.getPlaylist(uri, null,
            { offset: 0, limit: pageSize });
        const items = [...(first.contents?.items || [])];
        const total = first.contents?.totalLength ?? first.metadata?.totalLength ?? items.length;
        for (let offset = items.length; offset < total; offset += pageSize) {
            const page = await Spicetify.Platform.PlaylistAPI.getPlaylist(uri, null,
                { offset, limit: pageSize });
            const next = page.contents?.items || [];
            if (!next.length) break;
            items.push(...next);
        }
        return {
            uri,
            name: first.metadata?.name ?? fallbackName ?? "Playlist",
            liked,
            image: (first.metadata?.images && first.metadata.images[0]?.url) || null,
            tracks: items.filter(t => t.type === "track" && !t.uri?.startsWith("spotify:local:")).map(t => ({
                uid: t.uid ?? null,
                uri: t.uri,
                name: t.name ?? null,
                album: t.album ? { uri: t.album.uri ?? null, name: t.album.name ?? null,
                    image: (t.album.images && t.album.images[0]?.url) || null } : null,
                artists: (t.artists || []).map(a => ({ uri: a.uri ?? null, name: a.name ?? null })),
                ms: t.duration?.totalMilliseconds ?? t.duration?.milliseconds ?? null,
                disc: t.discNumber ?? 0,
                num: t.trackNumber ?? 0
            }))
        };
    }

    async function dumpAlbum(uri, fallbackName) {
        const r = await Spicetify.GraphQL.Request(Spicetify.GraphQL.Definitions.getAlbum, { uri, offset: 0, limit: 2000 });
        const a = r?.data?.albumUnion;
        if (!a) throw new Error("no album data");
        const image = (a.coverArt?.sources && a.coverArt.sources[0]?.url) || null;
        return {
            uri: a.uri ?? uri,
            name: a.name ?? fallbackName ?? "Album",
            year: a.date?.year ?? null,
            image,
            artists: (a.artists?.items || []).map(x => ({ uri: x.uri ?? null, name: x.name ?? null })),
            tracks: (a.tracksV2?.items || []).filter(i => i.track).map(i => ({
                uri: i.track.uri,
                name: i.track.name ?? null,
                album: { uri, name: a.name ?? fallbackName ?? null, image },
                artists: (i.track.artists?.items || []).map(x => ({ uri: x.uri ?? null, name: x.name ?? null })),
                ms: i.track.duration?.totalMilliseconds ?? i.track.duration?.milliseconds ?? null,
                disc: i.track.discNumber ?? 0,
                num: i.track.trackNumber ?? 0
            }))
        };
    }

    function trackList(contents) {
        return (contents?.items || []).filter(t => t.type === "track" && !t.uri?.startsWith("spotify:local:")).map(t => ({
            uri: t.uri,
            name: t.name ?? null,
            album: t.album ? { uri: t.album.uri ?? null, name: t.album.name ?? null,
                image: (t.album.images && t.album.images[0]?.url) || null } : null,
            artists: (t.artists || []).map(a => ({ uri: a.uri ?? null, name: a.name ?? null })),
            ms: t.duration?.totalMilliseconds ?? t.duration?.milliseconds ?? null,
            disc: t.discNumber ?? 0,
            num: t.trackNumber ?? 0
        }));
    }

    const out = { playlists: [], virtual_albums: [], albums: [], artists: [], errors: [] };
    const cardImage = a => {
        for (let node = a; node; node = node.parentElement) {
            const image = node.querySelector("header img")?.src || node.querySelector("img")?.src;
            if (image) return image;
        }
        return null;
    };
    const readHomeLinks = () => [...document.querySelectorAll("a[href]")].map(a => ({
        name: (a.innerText || "").trim(), href: a.href,
        image: cardImage(a)
    })).filter(x => x.href.includes("/playlist/"));
    let homeLinks = readHomeLinks();
    if (!homeLinks.some(x => /^Daily Mix \d+$/i.test(x.name))) {
        const home = [...document.querySelectorAll("a,button,[role=button]")].find(x =>
            /^(home|accueil)$/i.test((x.innerText || x.getAttribute("aria-label") || "").trim())
        );
        home?.click();
        await new Promise(resolve => setTimeout(resolve, 3000));
        homeLinks = readHomeLinks();
    }
    const uri = href => "spotify:playlist:" + href.split("/playlist/")[1].split(/[?#]/)[0];
    const addVirtual = (name, links) => {
        const sources = [...new Set(links.map(x => uri(x.href)))];
        if (sources.length) out.virtual_albums.push({ name, sources, image: links.find(x => x.image)?.image || null });
    };
    for (const mix of homeLinks.filter(x => /^Daily Mix \d+$/i.test(x.name))) {
        addVirtual(mix.name, [mix]);
    }
    addVirtual("Discover Weekly", homeLinks.filter(x => /^Discover Weekly$/i.test(x.name)));
    for (const station of homeLinks.filter(x => / Radio$/i.test(x.name))) {
        addVirtual(station.name, [station]);
    }
    const biggestHits = homeLinks.filter(x => /Today.?s Biggest Hits/i.test(x.name));
    addVirtual("Today's Biggest Hits", biggestHits.length ? biggestHits : [
        { href: "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M" }
    ]);
    const likedUris = [
        Spicetify.Platform.LibraryAPI._likedSongsUri,
        "spotify:user:me:collection",
    ].filter((uri, index, all) => uri && all.indexOf(uri) === index);
    const root = await Spicetify.Platform.RootlistAPI.getContents({ offset: 0, limit: 500 });
    for (const entry of root.items) {
        if (entry.type !== "playlist") continue;
        try { out.playlists.push(await dumpPlaylist(entry.uri, entry.name)); }
        catch (e) { out.errors.push(`playlist ${entry.uri}: ${e}`); }
    }
    for (const uri of likedUris) {
        try {
            const liked = await dumpPlaylist(uri, "Liked Songs", true);
            if (liked.tracks.length || uri === likedUris[likedUris.length - 1]) {
                liked.name = "Liked Songs";
                out.playlists.push(liked);
                break;
            }
        } catch (e) { out.errors.push(`liked songs ${uri}: ${e}`); }
    }

    try {
        const library = await Spicetify.Platform.LibraryAPI.getContents({ offset: 0, limit: 500 });
        for (const row of library.items || []) {
            if (row.type === "album") {
                try { out.albums.push(await dumpAlbum(row.uri, row.name)); }
                catch (e) { out.errors.push(`album ${row.uri}: ${e}`); }
            } else if (row.type === "artist") {
                out.artists.push({ uri: row.uri, name: row.name, image: null });
            }
        }
    } catch (e) {}

    return JSON.stringify(out);
})()
"#;

pub async fn collect(bridge: &crate::bridge::BridgeState, cache_dir: &Path) -> Result<Catalog, String> {
    let response = eval_on_bridge(bridge, COLLECT_JS.to_string())
        .await
        .map_err(|error| format!("collect failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "collect returned no value".to_string())?;
    let playlists = serde_json::from_str::<Value>(raw)
        .map_err(|error| format!("collect JSON invalid: {error}"))?;
    if let Some(errors) = playlists.get("errors").and_then(Value::as_array) {
        for error in errors.iter().filter_map(Value::as_str) {
            eprintln!("catalog collector: {error}");
        }
    }
    let catalog = parse_catalog(&playlists)?;
    if let Some(items) = playlists.get("playlists").and_then(Value::as_array) {
        for playlist in items {
            if let Err(error) = cache_playlist(cache_dir, playlist).await {
                eprintln!("could not cache playlist: {error}");
            }
        }
    }
    Ok(catalog)
}

/// Restores playlists from the previous session while the Spotify renderer is
/// still starting. Invalid or incomplete cache files are ignored individually.
pub async fn load_playlist_cache(cache_dir: &Path) -> Catalog {
    let mut catalog = Catalog::new();
    let Ok(mut files) = tokio::fs::read_dir(cache_dir).await else {
        return catalog;
    };
    while let Ok(Some(entry)) = files.next_entry().await {
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                (name.starts_with("playlist-") || name.starts_with("track-"))
                    && name.ends_with(".json")
            })
        {
            continue;
        }
        let Ok(raw) = tokio::fs::read_to_string(&path).await else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if value.get("uri").and_then(Value::as_str).is_some() {
            let remote = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("track-"));
            let id = catalog::stable_id(value.get("uri").and_then(Value::as_str).unwrap());
            if remote {
                if ingest_track(&mut catalog, &value).is_some() {
                    catalog.mark_remote_track(id);
                }
            } else {
                add_playlist(&mut catalog, &value);
            }
        }
    }
    catalog
}

/// Persists the renderer's raw playlist shape so it can be restored without
/// waiting for the client to become available.
pub async fn cache_playlist(cache_dir: &Path, raw: &Value) -> Result<(), String> {
    let uri = raw
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "playlist has no URI".to_string())?;
    let id = catalog::stable_id(uri);
    let json = serde_json::to_vec_pretty(raw).map_err(|error| error.to_string())?;
    let path = cache_dir.join(format!("playlist-{id}.json"));
    let temporary = cache_dir.join(format!("playlist-{id}.json.tmp"));
    tokio::fs::write(&temporary, json).await.map_err(|error| error.to_string())?;
    tokio::fs::rename(&temporary, &path).await.map_err(|error| error.to_string())
}

/// Persists a search result separately from the browsable library. The raw
/// renderer shape is enough to reconstruct the stable item id and playback
/// metadata after a backend restart.
pub async fn cache_remote_track(cache_dir: &Path, raw: &Value) -> Result<(), String> {
    let uri = raw
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "search result has no URI".to_string())?;
    let id = catalog::stable_id(uri);
    let json = serde_json::to_vec_pretty(raw).map_err(|error| error.to_string())?;
    let path = cache_dir.join(format!("track-{id}.json"));
    let temporary = cache_dir.join(format!("track-{id}.json.tmp"));
    tokio::fs::write(&temporary, json).await.map_err(|error| error.to_string())?;
    tokio::fs::rename(&temporary, &path).await.map_err(|error| error.to_string())
}

/// Persists the complete track list fetched for an album. This keeps albums
/// discovered from playback usable after a backend restart.
pub async fn cache_album_tracks(cache_dir: &Path, album_uri: &str, tracks: &[Value]) -> Result<(), String> {
    let id = catalog::stable_id(album_uri);
    let raw = serde_json::json!({ "uri": album_uri, "tracks": tracks });
    let json = serde_json::to_vec_pretty(&raw).map_err(|error| error.to_string())?;
    let path = cache_dir.join(format!("album-{id}.json"));
    let temporary = cache_dir.join(format!("album-{id}.json.tmp"));
    tokio::fs::write(&temporary, json).await.map_err(|error| error.to_string())?;
    tokio::fs::rename(&temporary, &path).await.map_err(|error| error.to_string())
}

/// Reads a previously fetched album without adding it to the catalog. Album
/// caches are intentionally loaded lazily when a client opens the album.
pub async fn load_cached_album(cache_dir: &Path, album_uri: &str) -> Option<Vec<Value>> {
    let id = catalog::stable_id(album_uri);
    let path = cache_dir.join(format!("album-{id}.json"));
    let raw = tokio::fs::read_to_string(path).await.ok()?;
    let value = serde_json::from_str::<Value>(&raw).ok()?;
    Some(value.get("tracks")?.as_array()?.clone())
}

const SEARCH_JS: &str = r#"
(async () => {
    const query = QUERY_PLACEHOLDER;
    const r = await Spicetify.GraphQL.Request(Spicetify.GraphQL.Definitions.searchSuggestions, { query, limit: 20 });
    const items = r?.data?.searchV2?.topResultsV2?.itemsV2 ?? [];
    const tracks = items.filter(i => i.item?.__typename === "TrackResponseWrapper").map(i => i.item.data);
    return JSON.stringify(tracks.filter(t => !t.uri?.startsWith("spotify:local:")).map(t => ({
        uri: t.uri,
        name: t.name ?? null,
        album: t.albumOfTrack ? { uri: t.albumOfTrack.uri ?? null, name: t.albumOfTrack.name ?? null,
            image: (t.albumOfTrack.coverArt?.sources && t.albumOfTrack.coverArt.sources[0]?.url) || null } : null,
        artists: ((t.artists?.items) || []).map(a => ({ uri: a.uri ?? null, name: a.profile?.name ?? a.name ?? null })),
        ms: t.duration?.totalMilliseconds ?? t.duration?.milliseconds ?? null,
        disc: 1,
        num: 0
    })));
})()
"#;

const AUTOPLAY_JS: &str = r#"
(async () => {
    const seed = SEED_PLACEHOLDER;
    if (seed) {
        const station = seed.startsWith("spotify:track:")
            ? "spotify:station:track:" + seed.slice("spotify:track:".length)
            : seed;
        await Spicetify.Player.playUri(station);
        await new Promise(resolve => setTimeout(resolve, 5000));
    }
    const tracks = Spicetify.Player.data?.nextItems || [];
    return JSON.stringify(tracks.filter(t => t?.type === "track" && t?.uri).map(t => ({
        uri: t.uri,
        name: t.name ?? null,
        album: t.album ? {
            uri: t.album.uri ?? null,
            name: t.album.name ?? null,
            image: (t.album.images && t.album.images[0]?.url) || null
        } : null,
        artists: (t.artists || []).map(a => ({ uri: a.uri ?? null, name: a.name ?? null })),
        ms: t.duration?.totalMilliseconds ?? t.duration?.milliseconds ?? null,
        disc: Number(t.metadata?.album_disc_number ?? 0),
        num: Number(t.metadata?.album_track_number ?? 0)
    })));
})()
"#;

/// Returns Spotify's currently materialized autoplay recommendations. The
/// renderer already has complete metadata for these tracks, so no public API
/// request is needed.
pub async fn autoplay_tracks(bridge: &BridgeState, seed_uri: Option<&str>) -> Result<Vec<Value>, String> {
    let seed = seed_uri
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| "null".to_string());
    let code = AUTOPLAY_JS.replace("SEED_PLACEHOLDER", &seed);
    let response = eval_on_bridge(bridge, code)
        .await
        .map_err(|error| format!("autoplay query failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "autoplay query returned no value".to_string())?;
    serde_json::from_str::<Vec<Value>>(raw).map_err(|error| format!("autoplay JSON invalid: {error}"))
}

/// Searches Spotify from the renderer and returns raw track entries ready
/// for `ingest_track`.
pub async fn search(bridge: &BridgeState, query: &str) -> Result<Vec<Value>, String> {
    let literal = serde_json::to_string(query).map_err(|e| e.to_string())?;
    let code = SEARCH_JS.replace("QUERY_PLACEHOLDER", &literal);
    let response = eval_on_bridge(bridge, code)
        .await
        .map_err(|error| format!("search failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "search returned no value".to_string())?;
    serde_json::from_str::<Vec<Value>>(raw).map_err(|error| format!("search JSON invalid: {error}"))
}

const ARTIST_TRACKS_JS: &str = r#"
(async () => {
    const artistUri = ARTIST_URI_PLACEHOLDER;
    const overview = await Spicetify.GraphQL.Request(
        Spicetify.GraphQL.Definitions.queryArtistOverview,
        { uri: artistUri }
    );
    const image = overview?.data?.artistUnion?.visuals?.avatarImage?.sources?.[0]?.url || null;
    const releases = await Spicetify.GraphQL.Request(
        Spicetify.GraphQL.Definitions.queryArtistDiscographyAll,
        { uri: artistUri, offset: 0, limit: 2000 }
    );
    const albums = (releases?.data?.artistUnion?.discography?.all?.items || [])
        .flatMap(group => group.releases?.items || []);
    const tracks = [];
    for (const release of albums) {
        const result = await Spicetify.GraphQL.Request(
            Spicetify.GraphQL.Definitions.getAlbum,
            { uri: release.uri, offset: 0, limit: 2000 }
        );
        const album = result?.data?.albumUnion;
        if (!album) continue;
        const image = (album.coverArt?.sources && album.coverArt.sources[0]?.url) || null;
        for (const row of album.tracksV2?.items || []) {
            const track = row.track;
            if (!track?.uri) continue;
            tracks.push({
                uri: track.uri,
                name: track.name ?? null,
                album: { uri: album.uri ?? release.uri, name: album.name ?? release.name, image,
                    year: release.date?.year ?? album.date?.year ?? null },
                artists: (track.artists?.items || []).map(a => ({
                    uri: a.uri ?? null, name: a.profile?.name ?? a.name ?? null
                })),
                ms: track.duration?.totalMilliseconds ?? track.duration?.milliseconds ?? null,
                disc: track.discNumber ?? 0,
                num: track.trackNumber ?? 0
            });
        }
    }
    return JSON.stringify({ tracks, image });
})()
"#;

pub struct ArtistFetch {
    pub tracks: Vec<Value>,
    pub image: Option<String>,
}

pub async fn artist_tracks(bridge: &BridgeState, artist_uri: &str) -> Result<ArtistFetch, String> {
    let literal = serde_json::to_string(artist_uri).map_err(|e| e.to_string())?;
    let code = ARTIST_TRACKS_JS.replace("ARTIST_URI_PLACEHOLDER", &literal);
    let response = eval_on_bridge(bridge, code).await.map_err(|error| format!("artist fetch failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "artist fetch returned no value".to_string())?;
    serde_json::from_str::<ArtistFetchRaw>(raw)
        .map(|result| ArtistFetch { tracks: result.tracks, image: result.image })
        .map_err(|error| format!("artist JSON invalid: {error}"))
}

const ALBUM_TRACKS_JS: &str = r#"
(async () => {
    const result = await Spicetify.GraphQL.Request(
        Spicetify.GraphQL.Definitions.getAlbum,
        { uri: ALBUM_URI_PLACEHOLDER, offset: 0, limit: 2000 }
    );
    const album = result?.data?.albumUnion;
    if (!album) throw new Error("no album data");
    const image = album.coverArt?.sources?.[0]?.url || null;
    return JSON.stringify((album.tracksV2?.items || []).filter(row => row.track?.uri).map(row => ({
        uri: row.track.uri,
        name: row.track.name ?? null,
        album: { uri: album.uri ?? ALBUM_URI_PLACEHOLDER, name: album.name ?? null, image,
            year: album.date?.year ?? null },
        artists: (row.track.artists?.items || []).map(a => ({
            uri: a.uri ?? null, name: a.profile?.name ?? a.name ?? null
        })),
        ms: row.track.duration?.totalMilliseconds ?? row.track.duration?.milliseconds ?? null,
        disc: row.track.discNumber ?? 0,
        num: row.track.trackNumber ?? 0
    })));
})()
"#;

pub async fn album_tracks(bridge: &BridgeState, album_uri: &str) -> Result<Vec<Value>, String> {
    let literal = serde_json::to_string(album_uri).map_err(|e| e.to_string())?;
    let code = ALBUM_TRACKS_JS.replace("ALBUM_URI_PLACEHOLDER", &literal);
    let response = eval_on_bridge(bridge, code).await.map_err(|error| format!("album fetch failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "album fetch returned no value".to_string())?;
    serde_json::from_str::<Vec<Value>>(raw).map_err(|error| format!("album JSON invalid: {error}"))
}

#[derive(serde::Deserialize)]
struct ArtistFetchRaw {
    tracks: Vec<Value>,
    image: Option<String>,
}

/// Inserts a raw track entry (collector or search shape) if new; either way
/// returns the track's stable id.
pub fn ingest_track(catalog: &mut Catalog, raw: &Value) -> Option<Uuid> {
    add_track(catalog, raw)
}

const LYRICS_JS: &str = r#"
(async () => {
    const id = URI_PLACEHOLDER.split(":").pop();
    const r = await Spicetify.CosmosAsync.get(
        "https://spclient.wg.spotify.com/color-lyrics/v2/track/" + id +
        "?format=json&vocalRemoval=false&market=from_token"
    );
    return JSON.stringify((r?.lyrics?.lines || []).map(l => {
        const words = l.syllables || l.wordsWithTimestamps;
        if (Array.isArray(words) && words.length > 0) {
            const text = l.words ?? "";
            let position = 0;
            const cues = words.map(word => {
                const cueText = word.syllable ?? word.word ?? word.text ?? "";
                const found = text.indexOf(cueText, position);
                const cuePosition = found >= 0 ? found : position;
                position = cuePosition + cueText.length;
                return {
                    position: cuePosition,
                    endPosition: position,
                    start: Number(word.startTimeMs ?? 0),
                    end: Number(word.endTimeMs ?? 0)
                };
            });
            return { start: Number(l.startTimeMs ?? 0), text, cues };
        }
        return { start: Number(l.startTimeMs ?? 0), text: l.words ?? "" };
    }));
})()
"#;

pub struct Lyric {
    pub start_ms: u64,
    pub text: String,
    pub cues: Option<Vec<LyricCue>>,
}

pub struct LyricCue {
    pub position: u32,
    pub end_position: u32,
    pub start_ms: u64,
    pub end_ms: Option<u64>,
}

pub async fn lyrics(bridge: &BridgeState, track_uri: &str) -> Result<Vec<Lyric>, String> {
    let literal = serde_json::to_string(track_uri).map_err(|e| e.to_string())?;
    let code = LYRICS_JS.replace("URI_PLACEHOLDER", &literal);
    let response = eval_on_bridge(bridge, code)
        .await
        .map_err(|error| format!("lyrics failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "lyrics returned no value".to_string())?;
    let lines = serde_json::from_str::<Value>(raw).map_err(|error| format!("lyrics JSON invalid: {error}"))?;
    Ok(lines
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(|line| {
            Some(Lyric {
                start_ms: line.get("start")?.as_u64()?,
                text: str_field(line, "text")?.to_string(),
                cues: line.get("cues").and_then(Value::as_array).map(|cues| {
                    cues.iter()
                        .filter_map(|cue| Some(LyricCue {
                            position: cue.get("position")?.as_u64()? as u32,
                            end_position: cue.get("endPosition")?.as_u64()? as u32,
                            start_ms: cue.get("start")?.as_u64()?,
                            end_ms: cue.get("end").and_then(Value::as_u64),
                        }))
                        .collect()
                }),
            })
        })
        .collect())
}

const SET_FAVORITE_JS: &str = r#"
(async () => {
    const uri = URI_PLACEHOLDER;
    const api = Spicetify.Platform.LibraryAPI;
    const method = LIKED_PLACEHOLDER ? api.add : api.remove;
    if (typeof method !== "function") throw new Error("Spotify LibraryAPI favorite method unavailable");
    await method.call(api, { uris: [uri] });
    return "ok";
})()
"#;

pub async fn set_favorite(bridge: &BridgeState, uri: &str, favorite: bool) -> Result<(), String> {
    let code = SET_FAVORITE_JS
        .replace("URI_PLACEHOLDER", &serde_json::to_string(uri).map_err(|e| e.to_string())?)
        .replace("LIKED_PLACEHOLDER", if favorite { "true" } else { "false" });
    eval_on_bridge(bridge, code).await.map_err(|e| format!("favorite update failed: {e}"))?;
    Ok(())
}

fn parse_catalog(raw: &Value) -> Result<Catalog, String> {
    let mut catalog = Catalog::new();
    for playlist in array_of(raw.get("playlists")) {
        add_playlist(&mut catalog, playlist);
        if playlist.get("liked").and_then(Value::as_bool) == Some(true) {
            for track in array_of(playlist.get("tracks")) {
                if let Some(uri) = str_field(track, "uri") {
                    catalog.set_favorite(catalog::stable_id(uri), true);
                }
            }
        }
    }
    for album in array_of(raw.get("albums")) {
        add_saved_album(&mut catalog, album);
    }
    for artist in array_of(raw.get("artists")) {
        if let (Some(uri), Some(name)) = (str_field(artist, "uri"), str_field(artist, "name")) {
            let id = catalog::stable_id(uri);
            catalog.artists.entry(id).or_insert(Artist {
                id,
                uri: uri.to_string(),
                name: name.to_string(),
                image: None,
                discography_loaded: false,
            });
            catalog.followed_artists.insert(id);
        }
    }
    for virtual_raw in array_of(raw.get("virtual_albums")) {
        let Some(name) = str_field(virtual_raw, "name") else { continue };
        let sources = array_of(virtual_raw.get("sources")).iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>();
        if sources.is_empty() { continue; }
        let id = catalog::stable_id(&format!("virtual-playlist:{name}"));
        catalog.playlists.insert(id, Playlist {
            id, name: name.to_string(), image: image_of(virtual_raw.get("image")), spotify_uri: None,
            source_uris: sources, loaded: false, entries: Vec::new(),
        });
    }
    Ok(catalog)
}

fn array_of(value: Option<&Value>) -> &[Value] {
    static EMPTY: [Value; 0] = [];
    value.and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&EMPTY)
}

fn add_saved_album(catalog: &mut Catalog, raw: &Value) {
    let Some(uri) = str_field(raw, "uri") else {
        return;
    };
    let id = catalog::stable_id(uri);
    let artist_ids = artist_ids(catalog, raw.get("artists"));
    let name = str_field(raw, "name").unwrap_or("Album").to_string();
    let year = raw.get("year").and_then(Value::as_i64).map(|value| value as i32);
    catalog.saved_albums.insert(id);
    match catalog.albums.get_mut(&id) {
        Some(album) => album.year = album.year.or(year),
        None => {
            catalog.albums.insert(id, Album {
                id,
                uri: uri.to_string(),
                name,
                artist_ids,
                image: image_of(raw.get("image")),
                year,
            });
        }
    }
    let empty = Vec::new();
    let tracks = raw.get("tracks").and_then(Value::as_array).unwrap_or(&empty);
    for track in tracks {
        add_track(catalog, track);
    }
}

pub(crate) fn add_playlist(catalog: &mut Catalog, raw: &Value) {
    let Some(uri) = str_field(raw, "uri") else {
        return;
    };
    let id = catalog::stable_id(uri);
    let mut entries = Vec::new();
    let empty = Vec::new();
    let tracks = raw.get("tracks").and_then(Value::as_array).unwrap_or(&empty);
    for (position, track_raw) in tracks.iter().enumerate() {
        if let Some(track_id) = add_track(catalog, track_raw) {
            let key = match str_field(track_raw, "uid") {
                Some(uid) => format!("{id}:spotify:{uid}"),
                None => format!("{id}:{position}"),
            };
            entries.push(PlaylistEntry {
                id: catalog::stable_id(&key),
                uid: str_field(track_raw, "uid").map(str::to_string),
                track_id,
            });
        }
    }
    catalog.playlists.insert(id, Playlist {
        id,
        name: str_field(raw, "name").unwrap_or("Unnamed playlist").to_string(),
        image: image_of(raw.get("image")),
        spotify_uri: Some(uri.to_string()),
        source_uris: Vec::new(),
        loaded: true,
        entries,
    });
}

fn add_track(catalog: &mut Catalog, raw: &Value) -> Option<Uuid> {
    let uri = str_field(raw, "uri")?;
    if uri.starts_with("spotify:local:") {
        return None;
    }
    let id = catalog::stable_id(uri);
    let artist_ids = artist_ids(catalog, raw.get("artists"));
    let album_id = raw.get("album").and_then(|album| add_album(catalog, album, &artist_ids));
    if catalog.tracks.contains_key(&id) {
        return Some(id);
    }
    catalog.tracks.insert(id, Track {
        id,
        uri: uri.to_string(),
        name: str_field(raw, "name").unwrap_or("Unknown track").to_string(),
        album_id,
        artist_ids,
        index: num_field(raw, "num"),
        disc: num_field(raw, "disc"),
        duration_ms: raw.get("ms").and_then(Value::as_u64).unwrap_or(0),
    });
    Some(id)
}

fn add_album(catalog: &mut Catalog, raw: &Value, artist_ids: &[Uuid]) -> Option<Uuid> {
    let uri = str_field(raw, "uri")?;
    let id = catalog::stable_id(uri);
    let year = raw.get("year").and_then(Value::as_i64).map(|year| year as i32);
    let album = catalog.albums.entry(id).or_insert_with(|| {
        let name = str_field(raw, "name").unwrap_or_default().to_string();
        // Local-file pseudo albums have empty names; fall back to a fixed label.
        let name = if name.is_empty() { "Local files".to_string() } else { name };
        Album {
            id,
            uri: uri.to_string(),
            name,
            artist_ids: artist_ids.to_vec(),
            image: image_of(raw.get("image")),
            year,
        }
    });
    if album.year.is_none() {
        album.year = year;
    }
    Some(id)
}

fn artist_ids(catalog: &mut Catalog, raw: Option<&Value>) -> Vec<Uuid> {
    let Some(list) = raw.and_then(Value::as_array) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|artist| {
            let uri = str_field(artist, "uri")?;
            let id = catalog::stable_id(uri);
            let name = str_field(artist, "name").unwrap_or("Unknown artist");
            if !name.is_empty() {
                catalog.artists.entry(id).or_insert(Artist {
                    id,
                    uri: uri.to_string(),
                    name: name.to_string(),
                    image: None,
                    discography_loaded: false,
                });
            }
            Some(id)
        })
        .collect()
}

fn image_of(raw: Option<&Value>) -> Option<String> {
    match raw {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn num_field(value: &Value, key: &str) -> u32 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0) as u32
}

const CREATE_PLAYLIST_JS: &str = r#"
(async () => {
    const name = NAME_PLACEHOLDER;
    const r = await Spicetify.Platform.RootlistAPI.applyModification({
        operation: "create", createItemKind: 1, name
    });
    if (!r?.success || !r.uri) throw new Error("creation refused");
    return r.uri;
})()
"#;

const MODIFY_JS: &str = r#"
(async () => {
    const uri = PLAYLIST_PLACEHOLDER;
    const api = Spicetify.Platform.PlaylistAPI;
    await ACTION_PLACEHOLDER;
    // The service applies mutations async; give the view time to settle.
    await new Promise(r => setTimeout(r, 2000));
    return "ok";
})()
"#;

/// Fetches one playlist in the collector's raw shape so `add_playlist` can
/// absorb it verbatim.
const DUMP_PLAYLIST_JS: &str = r#"
(async () => {
    const uri = PLAYLIST_PLACEHOLDER;
    const pageSize = 200;
    const p = await Spicetify.Platform.PlaylistAPI.getPlaylist(uri, null,
        { offset: 0, limit: pageSize });
    const items = [...(p.contents?.items || [])];
    const total = p.contents?.totalLength ?? p.metadata?.totalLength ?? items.length;
    for (let offset = items.length; offset < total; offset += pageSize) {
        const page = await Spicetify.Platform.PlaylistAPI.getPlaylist(uri, null,
            { offset, limit: pageSize });
        const next = page.contents?.items || [];
        if (!next.length) break;
        items.push(...next);
    }
    return JSON.stringify({
        uri,
        name: p.metadata?.name ?? null,
        image: (p.metadata?.images && p.metadata.images[0]?.url) || null,
        tracks: items.filter(t => t.type === "track" && !t.uri?.startsWith("spotify:local:")).map(t => ({
            uid: t.uid ?? null,
            uri: t.uri,
            name: t.name ?? null,
            album: t.album ? { uri: t.album.uri ?? null, name: t.album.name ?? null,
                image: (t.album.images && t.album.images[0]?.url) || null } : null,
            artists: (t.artists || []).map(a => ({ uri: a.uri ?? null, name: a.name ?? null })),
            ms: t.duration?.totalMilliseconds ?? t.duration?.milliseconds ?? null,
            disc: t.discNumber ?? 0,
            num: t.trackNumber ?? 0
        }))
    });
})()
"#;

fn playlist_uri_literal(spotify_uri: &str) -> String {
    serde_json::to_string(spotify_uri).unwrap_or_else(|_| "\"\"".to_string())
}

pub async fn create_playlist(bridge: &BridgeState, name: &str) -> Result<String, String> {
    let literal = serde_json::to_string(name).map_err(|e| e.to_string())?;
    let code = CREATE_PLAYLIST_JS.replace("NAME_PLACEHOLDER", &literal);
    eval_string(bridge, code).await
}

async fn eval_string(bridge: &BridgeState, code: String) -> Result<String, String> {
    let response = eval_on_bridge(bridge, code).await.map_err(|e| e.to_string())?;
    response
        .get("value")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "no value returned".to_string())
}

pub async fn add_tracks(bridge: &BridgeState, spotify_uri: &str, track_uris: &[String]) -> Result<(), String> {
    let uris = serde_json::to_string(track_uris).map_err(|e| e.to_string())?;
    let action = format!("api.add({},{},{{}})", playlist_uri_literal(spotify_uri), uris);
    run_modify(bridge, spotify_uri, &action).await
}

pub async fn remove_rows(bridge: &BridgeState, spotify_uri: &str, uids: &[String]) -> Result<(), String> {
    let rows: Vec<_> = uids.iter().map(|uid| serde_json::json!({"uid": uid})).collect();
    let rows = serde_json::to_string(&rows).map_err(|e| e.to_string())?;
    let action = format!("api.remove({},{},{{}})", playlist_uri_literal(spotify_uri), rows);
    run_modify(bridge, spotify_uri, &action).await
}

/// Moves one row so it ends up directly after `anchor_uid` ("start" for top).
pub async fn move_row(
    bridge: &BridgeState,
    spotify_uri: &str,
    uid: &str,
    anchor_after: Option<&str>,
) -> Result<(), String> {
    let anchor = match anchor_after {
        Some(uid) => serde_json::to_string(&serde_json::json!({"uid": uid})).map_err(|e| e.to_string())?,
        None => "\"start\"".to_string(),
    };
    let action = format!(
        "api.move({},[{{uid:{}}}],{})",
        playlist_uri_literal(spotify_uri),
        serde_json::to_string(uid).map_err(|e| e.to_string())?,
        anchor
    );
    run_modify(bridge, spotify_uri, &action).await
}

async fn run_modify(bridge: &BridgeState, spotify_uri: &str, action: &str) -> Result<(), String> {
    let code = MODIFY_JS
        .replace("PLAYLIST_PLACEHOLDER", &playlist_uri_literal(spotify_uri))
        .replace("ACTION_PLACEHOLDER", action);
    eval_on_bridge(bridge, code).await.map(|_| ()).map_err(|e| e.to_string())
}

/// Fetches one playlist from the client in collector shape; absorb it with
/// `absorb_playlist` without holding any lock across the await.
pub async fn fetch_playlist(bridge: &BridgeState, spotify_uri: &str) -> Result<Value, String> {
    let code = DUMP_PLAYLIST_JS.replace("PLAYLIST_PLACEHOLDER", &playlist_uri_literal(spotify_uri));
    let response = eval_on_bridge(bridge, code).await.map_err(|e| e.to_string())?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "resync returned no value".to_string())?;
    serde_json::from_str::<Value>(raw).map_err(|error| format!("resync JSON invalid: {error}"))
}

pub fn absorb_playlist(catalog: &mut Catalog, raw: &Value) {
    add_playlist(catalog, raw);
}

pub fn absorb_virtual_playlist(catalog: &mut Catalog, playlist_id: Uuid, raw: &Value) {
    let empty = Vec::new();
    let mut entries = Vec::new();
    for (position, track_raw) in raw.get("tracks").and_then(Value::as_array).unwrap_or(&empty).iter().enumerate() {
        if let Some(track_id) = add_track(catalog, track_raw) {
            if catalog.playlists.get(&playlist_id).is_some_and(|playlist| {
                playlist.entries.iter().any(|entry| entry.track_id == track_id)
            }) || entries.iter().any(|entry: &PlaylistEntry| entry.track_id == track_id) {
                continue;
            }
            entries.push(PlaylistEntry {
                id: catalog::stable_id(&format!("virtual:{playlist_id}:{position}:{track_id}")),
                uid: None,
                track_id,
            });
        }
    }
    if let Some(playlist) = catalog.playlists.get_mut(&playlist_id) {
        playlist.entries.extend(entries);
        playlist.loaded = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn playlist_cache_round_trip() {
        let cache_dir = std::env::temp_dir().join(format!(
            "spotify-server-playlist-cache-{}",
            std::process::id()
        ));
        let _ = tokio::fs::remove_dir_all(&cache_dir).await;
        tokio::fs::create_dir_all(&cache_dir).await.unwrap();
        let raw = serde_json::json!({
            "uri": "spotify:playlist:test-cache",
            "name": "Cached playlist",
            "image": null,
            "tracks": [{
                "uid": "row-1",
                "uri": "spotify:track:test-track",
                "name": "Cached track",
                "album": null,
                "artists": [],
                "ms": 1000,
                "disc": 1,
                "num": 1
            }]
        });

        cache_playlist(&cache_dir, &raw).await.unwrap();
        let restored = load_playlist_cache(&cache_dir).await;
        let playlist_id = catalog::stable_id("spotify:playlist:test-cache");
        let track_id = catalog::stable_id("spotify:track:test-track");
        let playlist = restored.playlists.get(&playlist_id).unwrap();
        assert_eq!(playlist.name, "Cached playlist");
        assert_eq!(playlist.entries.len(), 1);
        assert_eq!(playlist.entries[0].track_id, track_id);
        assert_eq!(restored.tracks.get(&track_id).unwrap().name, "Cached track");

        tokio::fs::remove_dir_all(&cache_dir).await.unwrap();
    }

    #[tokio::test]
    async fn album_cache_round_trip() {
        let cache_dir = std::env::temp_dir().join(format!("spotify-server-album-cache-{}", std::process::id()));
        let _ = tokio::fs::remove_dir_all(&cache_dir).await;
        tokio::fs::create_dir_all(&cache_dir).await.unwrap();
        let tracks = vec![serde_json::json!({
            "uri": "spotify:track:cached-album-track",
            "name": "Cached album track",
            "album": { "uri": "spotify:album:cached-album", "name": "Cached album", "image": null },
            "artists": [],
            "ms": 1000,
            "disc": 1,
            "num": 1
        })];

        cache_album_tracks(&cache_dir, "spotify:album:cached-album", &tracks).await.unwrap();
        let restored = load_cached_album(&cache_dir, "spotify:album:cached-album").await.unwrap();
        assert_eq!(restored, tracks);

        tokio::fs::remove_dir_all(&cache_dir).await.unwrap();
    }
}
