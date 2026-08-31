use serde_json::Value;
use uuid::Uuid;

use crate::bridge::{eval_on_bridge, BridgeState};
use crate::catalog::{self, Album, Artist, Catalog, Playlist, PlaylistEntry, Track};

/// One eval per refresh: pull playlists (rootlist + liked songs), then saved
/// albums with their tracks and followed artists. Runs inside the Spotify
/// renderer, so it is the client querying its own backend.
const COLLECT_JS: &str = r#"
(async () => {
    async function dumpPlaylist(uri, fallbackName) {
        const p = await Spicetify.Platform.PlaylistAPI.getPlaylist(uri);
        return {
            uri,
            name: p.metadata?.name ?? fallbackName ?? "Playlist",
            image: (p.metadata?.images && p.metadata.images[0]?.url) || null,
            tracks: trackList(p.contents)
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
                ms: i.track.duration?.milliseconds ?? null,
                disc: i.track.discNumber ?? 0,
                num: i.track.trackNumber ?? 0
            }))
        };
    }

    function trackList(contents) {
        return (contents?.items || []).filter(t => t.type === "track").map(t => ({
            uri: t.uri,
            name: t.name ?? null,
            album: t.album ? { uri: t.album.uri ?? null, name: t.album.name ?? null,
                image: (t.album.images && t.album.images[0]?.url) || null } : null,
            artists: (t.artists || []).map(a => ({ uri: a.uri ?? null, name: a.name ?? null })),
            ms: t.duration?.milliseconds ?? null,
            disc: t.discNumber ?? 0,
            num: t.trackNumber ?? 0
        }));
    }

    const out = { playlists: [], albums: [], artists: [] };
    const root = await Spicetify.Platform.RootlistAPI.getContents({ offset: 0, limit: 500 });
    for (const entry of root.items) {
        if (entry.type !== "playlist") continue;
        try { out.playlists.push(await dumpPlaylist(entry.uri, entry.name)); } catch (e) {}
    }
    try { out.playlists.push(await dumpPlaylist("spotify:user:me:collection", "Liked Songs")); } catch (e) {}

    try {
        const library = await Spicetify.Platform.LibraryAPI.getContents({ offset: 0, limit: 500 });
        for (const row of library.items || []) {
            if (row.type === "album") {
                try { out.albums.push(await dumpAlbum(row.uri, row.name)); } catch (e) {}
            } else if (row.type === "artist") {
                out.artists.push({ uri: row.uri, name: row.name });
            }
        }
    } catch (e) {}

    return JSON.stringify(out);
})()
"#;

pub async fn collect(bridge: &crate::bridge::BridgeState) -> Result<Catalog, String> {
    let response = eval_on_bridge(bridge, COLLECT_JS.to_string())
        .await
        .map_err(|error| format!("collect failed: {error}"))?;
    let raw = response
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "collect returned no value".to_string())?;
    let playlists = serde_json::from_str::<Value>(raw)
        .map_err(|error| format!("collect JSON invalid: {error}"))?;
    parse_catalog(&playlists)
}

const SEARCH_JS: &str = r#"
(async () => {
    const query = QUERY_PLACEHOLDER;
    const r = await Spicetify.GraphQL.Request(Spicetify.GraphQL.Definitions.searchSuggestions, { query, limit: 20 });
    const items = r?.data?.searchV2?.topResultsV2?.itemsV2 ?? [];
    const tracks = items.filter(i => i.item?.__typename === "TrackResponseWrapper").map(i => i.item.data);
    return JSON.stringify(tracks.map(t => ({
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

/// Inserts a raw track entry (collector or search shape) if new; either way
/// returns the track's stable id.
pub fn ingest_track(catalog: &mut Catalog, raw: &Value) -> Option<Uuid> {
    add_track(catalog, raw)
}

fn parse_catalog(raw: &Value) -> Result<Catalog, String> {
    let mut catalog = Catalog::new();
    for playlist in array_of(raw.get("playlists")) {
        add_playlist(&mut catalog, playlist);
    }
    for album in array_of(raw.get("albums")) {
        add_saved_album(&mut catalog, album);
    }
    for artist in array_of(raw.get("artists")) {
        if let (Some(uri), Some(name)) = (str_field(artist, "uri"), str_field(artist, "name")) {
            let id = catalog::stable_id(uri);
            catalog.artists.entry(id).or_insert(Artist { id, name: name.to_string() });
            catalog.followed_artists.insert(id);
        }
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

fn add_playlist(catalog: &mut Catalog, raw: &Value) {
    let Some(uri) = str_field(raw, "uri") else {
        return;
    };
    let id = catalog::stable_id(uri);
    let mut entries = Vec::new();
    let empty = Vec::new();
    let tracks = raw.get("tracks").and_then(Value::as_array).unwrap_or(&empty);
    for (position, track_raw) in tracks.iter().enumerate() {
        if let Some(track_id) = add_track(catalog, track_raw) {
            let key = format!("{id}:{position}");
            entries.push(PlaylistEntry { id: catalog::stable_id(&key), track_id });
        }
    }
    catalog.playlists.insert(id, Playlist {
        id,
        name: str_field(raw, "name").unwrap_or("Unnamed playlist").to_string(),
        image: image_of(raw.get("image")),
        entries,
    });
}

fn add_track(catalog: &mut Catalog, raw: &Value) -> Option<Uuid> {
    let uri = str_field(raw, "uri")?;
    let id = catalog::stable_id(uri);
    if catalog.tracks.contains_key(&id) {
        return Some(id);
    }
    let artist_ids = artist_ids(catalog, raw.get("artists"));
    let album_id = raw.get("album").and_then(|album| add_album(catalog, album, &artist_ids));
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
    if !catalog.albums.contains_key(&id) {
        let name = str_field(raw, "name").unwrap_or_default().to_string();
        // Local-file pseudo albums have empty names; fall back to a fixed label.
        let name = if name.is_empty() { "Local files".to_string() } else { name };
        catalog.albums.insert(id, Album {
            id,
            name,
            artist_ids: artist_ids.to_vec(),
            image: image_of(raw.get("image")),
            year: None,
        });
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
                catalog.artists.entry(id).or_insert(Artist { id, name: name.to_string() });
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
