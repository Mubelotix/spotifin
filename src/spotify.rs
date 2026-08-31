use serde_json::Value;
use uuid::Uuid;

use crate::bridge::eval_on_bridge;
use crate::catalog::{self, Album, Artist, Catalog, Playlist, PlaylistEntry, Track};

/// One eval per refresh: walk the rootlist, then pull every playlist with its
/// tracks. Runs inside the Spotify renderer, so it is the client querying
/// its own backend.
const COLLECT_JS: &str = r#"
(async () => {
    const root = await Spicetify.Platform.RootlistAPI.getContents({ offset: 0, limit: 500 });
    const out = [];
    for (const entry of root.items) {
        if (entry.type !== "playlist") continue;
        const playlist = await Spicetify.Platform.PlaylistAPI.getPlaylist(entry.uri);
        const items = (playlist.contents?.items || []).filter(t => t.type === "track");
        out.push({
            uri: entry.uri,
            name: entry.name,
            image: (entry.images && entry.images[0]?.url) || null,
            totalLength: (playlist.contents?.totalLength) ?? null,
            tracks: items.map(t => ({
                uri: t.uri,
                name: t.name ?? null,
                album: t.album ? { uri: t.album.uri ?? null, name: t.album.name ?? null,
                    image: (t.album.images && t.album.images[0]?.url) || null } : null,
                artists: (t.artists || []).map(a => ({ uri: a.uri ?? null, name: a.name ?? null })),
                ms: t.duration?.milliseconds ?? null,
                disc: t.discNumber ?? 0,
                num: t.trackNumber ?? 0
            }))
        });
    }
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

fn parse_catalog(raw: &Value) -> Result<Catalog, String> {
    let entries = raw.as_array().ok_or("expected playlist array")?;
    let mut catalog = Catalog::new();
    for playlist in entries {
        add_playlist(&mut catalog, playlist);
    }
    Ok(catalog)
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
            catalog.artists.entry(id).or_insert_with(|| Artist {
                id,
                name: str_field(artist, "name").unwrap_or("Unknown artist").to_string(),
            });
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
