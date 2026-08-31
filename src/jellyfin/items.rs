use rocket::http::Status;
use serde_json::Value;
use rocket::serde::json::Json;
use rocket::FromForm;
use rocket::{get, State};
use uuid::Uuid;

use crate::catalog;
use crate::jellyfin::dto::{base_item, BaseItemDto, QueryResult};
use crate::AppState;

#[derive(FromForm)]
pub struct ItemQuery {
    #[field(name = "IncludeItemTypes")]
    include_item_types: Option<String>,
    #[field(name = "ParentId")]
    parent_id: Option<String>,
    #[field(name = "SearchTerm")]
    search_term: Option<String>,
    #[field(name = "searchTerm")]
    search_term_lowercase: Option<String>,
    #[field(name = "Filters")]
    filters: Option<String>,
    #[field(name = "ArtistIds")]
    artist_ids: Option<String>,
    #[field(name = "AlbumArtistIds")]
    album_artist_ids: Option<String>,
    #[field(name = "Ids")]
    ids: Option<String>,
    #[field(name = "StartIndex")]
    start_index: Option<usize>,
    #[field(name = "Limit")]
    limit: Option<usize>,
}

fn parse_types(raw: Option<String>) -> Vec<&'static str> {
    const KNOWN: &[(&str, &str)] = &[
        ("Audio", "Audio"),
        ("MusicAlbum", "MusicAlbum"),
        ("MusicArtist", "MusicArtist"),
        ("MusicGenre", "MusicGenre"),
        ("Playlist", "Playlist"),
        ("CollectionFolder", "CollectionFolder"),
        ("Folder", "CollectionFolder"),
    ];
    raw.map(|list| {
        list.split(',')
            .filter_map(|kind| KNOWN.iter().find(|(name, _)| name.eq_ignore_ascii_case(kind.trim())).map(|(_, jellyfin)| *jellyfin))
            .collect()
    })
    .unwrap_or_default()
}

fn page<T>(items: Vec<T>, start: usize, limit: Option<usize>) -> (Vec<T>, usize) {
    let total = items.len();
    let sliced: Vec<T> = items
        .into_iter()
        .skip(start)
        .take(limit.unwrap_or(usize::MAX))
        .collect();
    (sliced, total)
}

fn parse_ids(raw: Option<&str>) -> Vec<Uuid> {
    raw.unwrap_or_default()
        .split(',')
        .filter_map(|id| Uuid::parse_str(id.trim()).ok())
        .collect()
}

async fn run_query(state: &AppState, query: ItemQuery) -> QueryResult<BaseItemDto> {
    let types = parse_types(query.include_item_types.clone());
    let parent = query.parent_id.as_deref().and_then(|raw| Uuid::parse_str(raw).ok());
    let search = query
        .search_term
        .as_deref()
        .or(query.search_term_lowercase.as_deref())
        .unwrap_or_default();
    let mut artist_ids = parse_ids(query.artist_ids.as_deref());
    let album_artist_ids = parse_ids(query.album_artist_ids.as_deref());
    let ids = parse_ids(query.ids.as_deref());
    if let Some(id) = parent {
        let is_artist = state.catalog.read().unwrap().artists.contains_key(&id);
        if is_artist && !artist_ids.contains(&id) {
            artist_ids.push(id);
        }
    }
    let mut fetch_artist_ids = artist_ids.clone();
    for id in &album_artist_ids {
        if !fetch_artist_ids.contains(id) {
            fetch_artist_ids.push(*id);
        }
    }
    let favorites_only = query
        .filters
        .as_deref()
        .map(|filters| filters.split(',').any(|filter| filter.trim().eq_ignore_ascii_case("IsFavorite")))
        .unwrap_or(false);

    // Synthesized Spotify playlists are metadata-only until their page is
    // opened. Fetching each backing playlist here keeps normal refreshes cheap.
    if let Some(parent_id) = parent {
        let sources = state.catalog.read().unwrap().playlists.get(&parent_id).and_then(|playlist| {
            (!playlist.loaded).then(|| playlist.source_uris.clone())
        });
        if let Some(sources) = sources {
            for source in sources {
                match crate::spotify::fetch_playlist(&state.bridge, &source).await {
                    Ok(raw) => crate::spotify::absorb_virtual_playlist(&mut state.catalog.write().unwrap(), parent_id, &raw),
                    Err(error) => eprintln!("virtual album fetch failed for {source}: {error}"),
                }
            }
        }
    }

    // Fetch an artist's complete discography before filtering, so artist pages
    // are not limited to saved albums or tracks previously seen by the server.
    for artist_id in &fetch_artist_ids {
        let uri = {
            let catalog = state.catalog.read().unwrap();
            catalog.artists.get(artist_id).map(|artist| artist.uri.clone())
        };
        if let Some(uri) = uri {
            match crate::spotify::artist_tracks(&state.bridge, &uri).await {
                Ok(results) => {
                    let mut catalog = state.catalog.write().unwrap();
                    for track in &results.tracks {
                        crate::spotify::ingest_track(&mut catalog, track);
                    }
                    if let Some(artist) = catalog.artists.get_mut(artist_id) {
                        artist.image = results.image;
                        artist.discography_loaded = true;
                    }
                }
                Err(error) => eprintln!("artist fetch failed for {uri}: {error}"),
            }
        }
    }

    // Remote search results are ingested before the local pass as well.
    if !search.is_empty() && parent.is_none() && artist_ids.is_empty() {
        match crate::spotify::search(&state.bridge, search).await {
            Ok(results) => {
                eprintln!("remote search {:?} -> {} results", search, results.len());
                {
                    let mut catalog = state.catalog.write().unwrap();
                    for track in &results {
                        if let Some(id) = crate::spotify::ingest_track(&mut catalog, track) {
                            catalog.mark_remote_track(id);
                        }
                    }
                }
                for track in &results {
                    if let Err(error) = crate::spotify::cache_remote_track(&state.audio.cache, track).await {
                        eprintln!("could not cache remote track: {error}");
                    }
                }
            }
            Err(error) => eprintln!("remote search failed: {error}"),
        }
    }

    let catalog = state.catalog.read().unwrap();
    let term = if search.is_empty() { None } else { Some(search) };
    let found = if ids.is_empty() {
        catalog.query(&types, parent, term, favorites_only, &artist_ids, &album_artist_ids)
    } else {
        ids.iter()
            .filter_map(|id| catalog.item(*id))
            .filter(|item| types.is_empty() || types.contains(&item.jellyfin_type()))
            .collect()
    };
    let start = query.start_index.unwrap_or(0);
    let (items, total) = page(found, start, query.limit);
    let dtos = items.iter().map(|item| base_item(&catalog, item, None)).collect();
    QueryResult { items: dtos, total_record_count: total, start_index: start }
}

#[get("/Users/<user_id>/Views")]
pub fn views(user_id: Uuid, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    if user_id != crate::jellyfin::auth::user_id() {
        return Json(QueryResult { items: vec![], total_record_count: 0, start_index: 0 });
    }
    view_result(state)
}

#[get("/UserViews")]
pub fn user_views(state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    view_result(state)
}

#[get("/Library/MediaFolders")]
pub fn media_folders(state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    view_result(state)
}

fn view_result(state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    let catalog = state.catalog.read().unwrap();
    let library = catalog.item(catalog::library_id()).unwrap();
    Json(QueryResult {
        items: vec![base_item(&catalog, &library, None)],
        total_record_count: 1,
        start_index: 0,
    })
}

#[get("/Users/<user_id>/Items?<query..>")]
pub async fn user_items(user_id: Uuid, query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    if user_id != crate::jellyfin::auth::user_id() {
        return Json(QueryResult { items: vec![], total_record_count: 0, start_index: 0 });
    }
    Json(run_query(state.inner(), query).await)
}

#[get("/Items?<query..>", rank = 2)]
pub async fn items(query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    Json(run_query(state.inner(), query).await)
}

type MaybeItem = Result<Json<BaseItemDto>, Status>;

/// The artist index is library-scoped: followed artists only, no remote search.
async fn artist_index(query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    let catalog = state.catalog.read().unwrap();
    let term = query.search_term.as_deref().filter(|term| !term.is_empty());
    let found = catalog.query(&["MusicArtist"], None, term, false, &[], &[]);
    let start = query.start_index.unwrap_or(0);
    let (items, total) = page(found, start, query.limit);
    let dtos = items.iter().map(|item| base_item(&catalog, item, None)).collect();
    Json(QueryResult { items: dtos, total_record_count: total, start_index: start })
}

#[get("/Artists/AlbumArtists?<query..>")]
pub async fn album_artists(query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    artist_index(query, state).await
}

#[get("/Artists/<name>")]
pub fn artist_by_name(name: &str, state: &State<AppState>) -> MaybeItem {
    let catalog = state.catalog.read().unwrap();
    let artist = catalog.artists.values().find(|artist| artist.name.eq_ignore_ascii_case(name)).ok_or(Status::NotFound)?;
    Ok(Json(base_item(&catalog, &crate::catalog::Item::Artist(artist), None)))
}

#[get("/Artists?<query..>", rank = 2)]
pub async fn all_artists(query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    artist_index(query, state).await
}

#[get("/Users/<user_id>/Items/<item_id>")]
pub fn user_item(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> MaybeItem {
    if user_id != crate::jellyfin::auth::user_id() {
        return Err(Status::NotFound);
    }
    item_detail(item_id, state)
}

#[get("/Items/<item_id>", rank = 3)]
pub fn item_detail(item_id: Uuid, state: &State<AppState>) -> MaybeItem {
    let catalog = state.catalog.read().unwrap();
    match catalog.item(item_id) {
        Some(item) => Ok(Json(base_item(&catalog, &item, None))),
        None => Err(Status::NotFound),
    }
}

#[get("/Items/<item_id>/InstantMix?<query..>")]
pub async fn instant_mix(item_id: Uuid, query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    let limit = query.limit.unwrap_or(50);
    let seed_uri = {
        let catalog = state.catalog.read().unwrap();
        match catalog.item(item_id) {
            Some(crate::catalog::Item::Track(track)) => Some(track.uri.clone()),
            _ => None,
        }
    };
    // A track mix is requested before the client starts its queue. Seed
    // Spotify first so nextItems belongs to the requested track, not to a
    // previous playback context.
    let seeded = seed_uri.is_some();
    let autoplay = crate::spotify::autoplay_tracks(&state.bridge, seed_uri.as_deref())
        .await
        .unwrap_or_default();

    if !autoplay.is_empty() {
        let mut catalog = state.catalog.write().unwrap();
        for track in autoplay.iter().take(limit) {
            crate::spotify::ingest_track(&mut catalog, track);
        }
    }

    let catalog = state.catalog.read().unwrap();
    let seed = catalog.item(item_id).map(|item| item.name().to_string());
    let mut tracks = if autoplay.is_empty() {
        if seeded {
            seed_uri
                .as_deref()
                .map(crate::catalog::stable_id)
                .and_then(|id| catalog.tracks.get(&id).map(crate::catalog::Item::Track))
                .into_iter()
                .collect()
        } else {
            catalog.random_tracks(limit)
        }
    } else {
        let mut ids = autoplay
            .iter()
            .take(limit)
            .filter_map(|raw| raw.get("uri").and_then(Value::as_str))
            .map(crate::catalog::stable_id)
            .collect::<Vec<_>>();
        if seeded {
            if let Some(seed_uri) = seed_uri.as_deref() {
                ids.insert(0, crate::catalog::stable_id(seed_uri));
                ids.truncate(limit);
            }
        }
        ids.into_iter()
            .filter_map(|id| catalog.tracks.get(&id).map(crate::catalog::Item::Track))
            .collect()
    };
    // The fallback keeps the old deterministic seed behavior. Spotify's
    // autoplay list is already ordered by its recommendation engine.
    if autoplay.is_empty() {
        if let Some(seed) = seed {
            tracks.sort_by_key(|track| format!("{}:{}", seed, track.id()));
        }
    }
    let dtos = tracks.iter().map(|track| base_item(&catalog, track, None)).collect();
    Json(QueryResult { items: dtos, total_record_count: tracks.len(), start_index: 0 })
}

#[get("/Items/<item_id>/Similar?<query..>")]
pub async fn similar(item_id: Uuid, query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    instant_mix(item_id, query, state).await
}

/// None of the client-facing data sources expose genres, so both families
/// answer with an empty result rather than 404.
fn empty_result() -> Json<QueryResult<BaseItemDto>> {
    Json(QueryResult { items: vec![], total_record_count: 0, start_index: 0 })
}

#[get("/Genres")]
pub fn genres() -> Json<QueryResult<BaseItemDto>> {
    empty_result()
}

#[get("/MusicGenres")]
pub fn music_genres() -> Json<QueryResult<BaseItemDto>> {
    empty_result()
}

/// Optional per TARGET.md; a few clients use it for type-as-you-search.
#[get("/Search/Hints?<query..>")]
pub async fn search_hints(query: ItemQuery, state: &State<AppState>) -> Json<Value> {
    let result = run_query(state, query).await;
    Json(serde_json::json!({
        "SearchHints": result.items,
        "TotalRecordCount": result.total_record_count,
    }))
}
