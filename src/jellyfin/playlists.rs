use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::FromForm;
use rocket::{delete, get, post, State};
use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

use crate::catalog::{self, Playlist};
use crate::jellyfin::dto::{base_item, QueryResult};
use crate::spotify;
use crate::AppState;

#[get("/Playlists?<query..>")]
pub fn playlists(query: PageQuery, state: &State<AppState>) -> Json<QueryResult<crate::jellyfin::dto::BaseItemDto>> {
    let catalog = state.catalog.read().unwrap();
    let mut items: Vec<_> = catalog.playlists.values().map(crate::catalog::Item::Playlist).collect();
    items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
    let total = items.len();
    let start = query.start_index.unwrap_or(0);
    let items = items
        .into_iter()
        .skip(start)
        .take(query.limit.unwrap_or(usize::MAX))
        .map(|item| crate::jellyfin::dto::base_item(&catalog, &item, None))
        .collect();
    Json(QueryResult { items, total_record_count: total, start_index: start })
}

async fn load_synthetic_playlist(playlist_id: Uuid, state: &State<AppState>) {
    let sources = state.catalog.read().unwrap().playlists.get(&playlist_id).and_then(|playlist| {
        (!playlist.loaded).then(|| playlist.source_uris.clone())
    });
    let Some(sources) = sources else { return };
    for source in sources {
        match spotify::fetch_playlist(&state.bridge, &source).await {
            Ok(raw) => spotify::absorb_virtual_playlist(&mut state.catalog.write().unwrap(), playlist_id, &raw),
            Err(error) => eprintln!("synthetic playlist fetch failed for {source}: {error}"),
        }
    }
}

#[get("/Playlists/<playlist_id>")]
pub async fn get_playlist(playlist_id: Uuid, state: &State<AppState>) -> Option<Json<Value>> {
    load_synthetic_playlist(playlist_id, state).await;
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

/// Jellify saves a playlist by sending its complete ordered contents.
#[post("/Playlists/<playlist_id>", format = "json", data = "<body>")]
pub async fn update_playlist(playlist_id: Uuid, body: Json<Value>, state: &State<AppState>) -> Result<Status, Status> {
    let name = body.0.get("Name").and_then(Value::as_str).map(str::to_string);
    let ids = body
        .0
        .get("Ids")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let (uri, track_uris) = {
        let catalog = state.catalog.read().unwrap();
        let playlist = catalog.playlists.get(&playlist_id).ok_or(Status::NotFound)?;
        let uri = playlist.spotify_uri.clone().ok_or(Status::Conflict)?;
        let tracks = ids.iter().filter_map(|id| Uuid::parse_str(id).ok()).filter_map(|id| catalog.tracks.get(&id).map(|track| track.uri.clone())).collect::<Vec<_>>();
        (uri, tracks)
    };
    if let Some(name) = name {
        spotify::rename_playlist(&state.bridge, &uri, &name).await.map_err(|_| Status::BadGateway)?;
    }
    spotify::replace_playlist(&state.bridge, &uri, &track_uris).await.map_err(|_| Status::BadGateway)?;
    resync(state, playlist_id).await;
    Ok(Status::NoContent)
}

/// Deletes a user-owned Spotify playlist from the rootlist.
#[delete("/Playlists/<playlist_id>")]
pub async fn delete_playlist(playlist_id: Uuid, state: &State<AppState>) -> Result<Status, Status> {
    let spotify_uri = state
        .catalog
        .read()
        .unwrap()
        .playlists
        .get(&playlist_id)
        .ok_or(Status::NotFound)?
        .spotify_uri
        .clone()
        .ok_or(Status::Conflict)?;
    spotify::delete_playlist(&state.bridge, &spotify_uri).await.map_err(|_| Status::BadGateway)?;
    state.catalog.write().unwrap().playlists.remove(&playlist_id);
    if let Err(error) = spotify::delete_playlist_cache(&state.audio.cache, playlist_id).await {
        eprintln!("could not remove deleted playlist cache: {error}");
    }
    Ok(Status::NoContent)
}

#[get("/Playlists/<playlist_id>/Items?<query..>")]
pub async fn playlist_items(
    playlist_id: Uuid,
    query: PageQuery,
    state: &State<AppState>,
) -> Json<QueryResult<crate::jellyfin::dto::BaseItemDto>> {
    load_synthetic_playlist(playlist_id, state).await;
    let catalog = state.catalog.read().unwrap();
    let start = query.start_index.unwrap_or(0);
    let Some(playlist) = catalog.playlists.get(&playlist_id) else {
        return Json(QueryResult { items: vec![], total_record_count: 0, start_index: 0 });
    };
    let total = playlist.entries.len();
    let items = playlist.entries
        .iter()
        .skip(start)
        .take(query.limit.unwrap_or(usize::MAX))
        .filter_map(|entry| catalog.tracks.get(&entry.track_id).map(|track| (track, entry)))
        .map(|(track, entry)| {
            let mut item = base_item(&catalog, &crate::catalog::Item::Track(track), Some(entry.id));
            item.parent_id = Some(playlist_id);
            item
        })
        .collect();
    Json(QueryResult { items, total_record_count: total, start_index: start })
}

#[derive(FromForm)]
pub struct PageQuery {
    #[field(name = "startindex")]
    start_index: Option<usize>,
    #[field(name = "limit")]
    limit: Option<usize>,
}

/// Creates the playlist inside Spotify itself; the returned URI is the
/// canonical identity of the new playlist.
#[post("/Playlists", format = "json", data = "<body>")]
pub async fn create_playlist(body: Json<Value>, state: &State<AppState>) -> Result<Json<Value>, Status> {
    let name = body.0.get("Name").and_then(Value::as_str).unwrap_or("New playlist").to_string();
    let initial_ids = body
        .0
        .get("Ids")
        .and_then(Value::as_array)
        .map(|ids| ids.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let uri = spotify::create_playlist(&state.bridge, &name).await.map_err(|_| Status::BadGateway)?;
    let id = catalog::stable_id(&uri);
    {
        let mut catalog = state.catalog.write().unwrap();
        let playlist = Playlist {
            id,
            name: name.clone(),
            image: None,
            spotify_uri: Some(uri.clone()),
            source_uris: Vec::new(),
            loaded: true,
            entries: Vec::new(),
        };
        catalog.playlists.entry(id).or_insert(playlist);
    }
    let raw = serde_json::json!({ "uri": uri, "name": name, "image": null, "tracks": [] });
    if let Err(error) = spotify::cache_playlist(&state.audio.cache, &raw).await {
        eprintln!("could not cache playlist: {error}");
    }
    if !initial_ids.is_empty() {
        let track_uris = {
            let catalog = state.catalog.read().unwrap();
            let track_ids: Vec<Uuid> = initial_ids.iter().filter_map(|id| Uuid::parse_str(id).ok()).collect();
            track_uris_of(&catalog, &track_ids)
        };
        if !track_uris.is_empty() {
            spotify::add_tracks(&state.bridge, &uri, &track_uris).await.map_err(|_| Status::BadGateway)?;
            resync(state, id).await;
        }
    }
    Ok(Json(serde_json::json!({ "Id": id })))
}

fn track_uris_of(catalog: &crate::catalog::Catalog, ids: &[Uuid]) -> Vec<String> {
    ids.iter().filter_map(|id| catalog.tracks.get(id).map(|t| t.uri.clone())).collect()
}

async fn resync(state: &State<AppState>, playlist_id: Uuid) {
    let uri = state.catalog.read().unwrap().playlists.get(&playlist_id).and_then(|p| p.spotify_uri.clone());
    let Some(uri) = uri else { return };
    match spotify::fetch_playlist(&state.bridge, &uri).await {
        Ok(raw) => {
            if let Err(error) = spotify::cache_playlist(&state.audio.cache, &raw).await {
                eprintln!("could not cache playlist: {error}");
            }
            spotify::absorb_playlist(&mut state.catalog.write().unwrap(), &raw);
        }
        Err(error) => eprintln!("resync failed for {uri}: {error}"),
    }
}

#[post("/Playlists/<playlist_id>/Items?<ids..>")]
pub async fn add_to_playlist(playlist_id: Uuid, ids: AddQuery, state: &State<AppState>) -> Result<Status, Status> {
    let (spotify_uri, uris) = {
        let catalog = state.catalog.read().unwrap();
        let playlist = catalog.playlists.get(&playlist_id).ok_or(Status::NotFound)?;
        let track_ids: Vec<Uuid> = ids
            .ids
            .iter()
            .filter_map(|raw| Uuid::parse_str(raw).ok())
            .collect();
        let existing: HashSet<&str> = playlist
            .entries
            .iter()
            .filter_map(|entry| catalog.tracks.get(&entry.track_id))
            .map(|track| track.uri.as_str())
            .collect();
        let mut added = HashSet::new();
        let uris = track_uris_of(&catalog, &track_ids)
            .into_iter()
            .filter(|uri| !existing.contains(uri.as_str()) && added.insert(uri.clone()))
            .collect::<Vec<_>>();
        (playlist.spotify_uri.clone(), uris)
    };
    if !uris.is_empty() {
        match spotify_uri {
            Some(uri) => spotify::add_tracks(&state.bridge, &uri, &uris).await.map_err(|_| Status::BadGateway)?,
            None => return Err(Status::Conflict), // ephemeral playlists have no client backing
        }
    }
    resync(state, playlist_id).await;
    Ok(Status::NoContent)
}

#[derive(FromForm)]
pub struct AddQuery {
    #[field(name = "ids")]
    ids: Vec<String>,
}

#[delete("/Playlists/<playlist_id>/Items?<entry_ids..>")]
pub async fn remove_from_playlist(
    playlist_id: Uuid,
    entry_ids: RemoveQuery,
    state: &State<AppState>,
) -> Result<Status, Status> {
    let (spotify_uri, uids) = {
        let catalog = state.catalog.read().unwrap();
        let playlist = catalog.playlists.get(&playlist_id).ok_or(Status::NotFound)?;
        let uids: Vec<String> = playlist
            .entries
            .iter()
            .filter(|entry| {
                let id = entry.id.to_string();
                entry_ids.entry_ids.contains(&id)
            })
            .filter_map(|entry| entry.uid.clone())
            .collect();
        (playlist.spotify_uri.clone(), uids)
    };
    if !uids.is_empty() {
        match spotify_uri {
            Some(uri) => spotify::remove_rows(&state.bridge, &uri, &uids).await.map_err(|_| Status::BadGateway)?,
            None => return Err(Status::Conflict),
        }
    } else if spotify_uri.is_none() {
        let mut catalog = state.catalog.write().unwrap();
        if let Some(playlist) = catalog.playlists.get_mut(&playlist_id) {
            playlist.entries.retain(|entry| {
                let id = entry.id.to_string();
                !entry_ids.entry_ids.contains(&id)
            });
        }
        return Ok(Status::NoContent);
    }
    resync(state, playlist_id).await;
    Ok(Status::NoContent)
}

#[derive(FromForm)]
pub struct RemoveQuery {
    #[field(name = "entryids")]
    entry_ids: Vec<String>,
}

/// Moves one entry to `new_index` with a single anchored client operation:
/// anchor after its final predecessor, or to the top when it becomes first.
#[post("/Playlists/<playlist_id>/Items/<entry_id>/Move/<new_index>")]
pub async fn move_entry(
    playlist_id: Uuid,
    entry_id: Uuid,
    new_index: usize,
    state: &State<AppState>,
) -> Result<Status, Status> {
    let plan = {
        let catalog = state.catalog.read().unwrap();
        let playlist = catalog.playlists.get(&playlist_id).ok_or(Status::NotFound)?;
        let old_position = playlist.entries.iter().position(|entry| entry.id == entry_id).ok_or(Status::NotFound)?;
        let uid = playlist.entries[old_position].uid.clone().ok_or(Status::Conflict)?;
        let rest: Vec<_> = playlist.entries.iter().filter(|entry| entry.id != entry_id).collect();
        let bounded = new_index.min(rest.len());
        let anchor_after = if bounded == 0 { None } else { rest.get(bounded - 1).and_then(|entry| entry.uid.clone()) };
        (playlist.spotify_uri.clone(), uid, anchor_after)
    };
    let (Some(spotify_uri), uid, anchor_after) = plan else {
        return Err(Status::Conflict);
    };
    spotify::move_row(&state.bridge, &spotify_uri, &uid, anchor_after.as_deref())
        .await
        .map_err(|_| Status::BadGateway)?;
    resync(state, playlist_id).await;
    Ok(Status::NoContent)
}
