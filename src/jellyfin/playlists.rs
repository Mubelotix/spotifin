use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::FromForm;
use rocket::{delete, get, post, State};
use serde_json::Value;
use uuid::Uuid;

use crate::catalog::{self, Playlist, PlaylistEntry};
use crate::jellyfin::dto::{base_item, QueryResult};
use crate::AppState;

#[get("/Playlists/<playlist_id>")]
pub fn get_playlist(playlist_id: Uuid, state: &State<AppState>) -> Option<Json<Value>> {
    let catalog = state.catalog.read().unwrap();
    let playlist = catalog.playlists.get(&playlist_id)?;
    Some(Json(serde_json::json!({
        "Id": playlist.id,
        "Name": playlist.name,
        "OpenAccess": false,
        "Shares": [],
        "ItemIds": catalog.playlist_tracks(playlist).iter().map(|t| t.id()).collect::<Vec<_>>(),
    })))
}

#[get("/Playlists/<playlist_id>/Items?<query..>")]
pub fn playlist_items(playlist_id: Uuid, query: PageQuery, state: &State<AppState>) -> Json<QueryResult<crate::jellyfin::dto::BaseItemDto>> {
    let catalog = state.catalog.read().unwrap();
    let start = query.start_index.unwrap_or(0);
    let Some(playlist) = catalog.playlists.get(&playlist_id) else {
        return Json(QueryResult { items: vec![], total_record_count: 0, start_index: 0 });
    };
    let tracks = catalog.playlist_tracks(playlist);
    let total = tracks.len();
    let items = tracks
        .into_iter()
        .zip(&playlist.entries)
        .skip(start)
        .take(query.limit.unwrap_or(usize::MAX))
        .map(|(track, entry)| base_item(&catalog, &track, Some(entry.id)))
        .collect();
    Json(QueryResult { items, total_record_count: total, start_index: start })
}

#[derive(FromForm)]
pub struct PageQuery {
    #[field(name = "StartIndex")]
    start_index: Option<usize>,
    #[field(name = "Limit")]
    limit: Option<usize>,
}

/// Creates a server-side empty playlist. Spotify-backed playlists are
/// read-only for now; new playlists live only in the in-memory catalog.
#[post("/Playlists", format = "json", data = "<body>")]
pub fn create_playlist(body: Json<Value>, state: &State<AppState>) -> Json<Value> {
    let name = body.0.get("Name").and_then(Value::as_str).unwrap_or("New playlist");
    let id = catalog::stable_id(&format!("playlist:new:{name}:{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)));
    let mut catalog = state.catalog.write().unwrap();
    catalog.playlists.insert(id, Playlist { id, name: name.to_string(), image: None, entries: Vec::new() });
    Json(serde_json::json!({ "Id": id }))
}

#[post("/Playlists/<playlist_id>/Items?<ids..>")]
pub fn add_to_playlist(playlist_id: Uuid, ids: AddQuery, state: &State<AppState>) -> Status {
    let mut catalog = state.catalog.write().unwrap();
    let track_ids: Vec<Uuid> = ids.ids.iter().filter_map(|raw| Uuid::parse_str(raw).ok()).collect();
    let Some(playlist) = catalog.playlists.get_mut(&playlist_id) else {
        return Status::NotFound;
    };
    let existing: Vec<_> = playlist.entries.iter().map(|entry| entry.track_id).collect();
    for (offset, track_id) in track_ids.into_iter().enumerate() {
        if existing.contains(&track_id) {
            continue;
        }
        let key = format!("{playlist_id}:new:{}", playlist.entries.len() + offset);
        playlist.entries.push(PlaylistEntry { id: catalog::stable_id(&key), track_id });
    }
    Status::NoContent
}

#[derive(FromForm)]
pub struct AddQuery {
    ids: Vec<String>,
}

#[delete("/Playlists/<playlist_id>/Items?<entry_ids..>")]
pub fn remove_from_playlist(playlist_id: Uuid, entry_ids: RemoveQuery, state: &State<AppState>) -> Status {
    let mut catalog = state.catalog.write().unwrap();
    let Some(playlist) = catalog.playlists.get_mut(&playlist_id) else {
        return Status::NotFound;
    };
    playlist.entries.retain(|entry| !entry_ids.entry_ids.contains(&entry.id.to_string()));
    Status::NoContent
}

#[derive(FromForm)]
pub struct RemoveQuery {
    #[field(name = "EntryIds")]
    entry_ids: Vec<String>,
}
