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

async fn run_query(state: &AppState, query: ItemQuery) -> QueryResult<BaseItemDto> {
    let types = parse_types(query.include_item_types.clone());
    let parent = query.parent_id.as_deref().and_then(|raw| Uuid::parse_str(raw).ok());
    let search = query.search_term.as_deref().unwrap_or_default();

    // Remote results are ingested before the local pass so they participate
    // in filtering, sorting and pagination exactly like library items.
    if !search.is_empty() && parent.is_none() {
        match crate::spotify::search(&state.bridge, search).await {
            Ok(results) => {
                eprintln!("remote search {:?} -> {} results", search, results.len());
                let mut catalog = state.catalog.write().unwrap();
                for track in &results {
                    crate::spotify::ingest_track(&mut catalog, track);
                }
            }
            Err(error) => eprintln!("remote search failed: {error}"),
        }
    }

    let catalog = state.catalog.read().unwrap();
    let term = if search.is_empty() { None } else { Some(search) };
    let found = catalog.query(&types, parent, term);
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
    let found = catalog.query(&["MusicArtist"], None, term);
    let start = query.start_index.unwrap_or(0);
    let (items, total) = page(found, start, query.limit);
    let dtos = items.iter().map(|item| base_item(&catalog, item, None)).collect();
    Json(QueryResult { items: dtos, total_record_count: total, start_index: start })
}

#[get("/Artists/AlbumArtists?<query..>")]
pub async fn album_artists(query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    artist_index(query, state).await
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

#[get("/Items/<item_id>/InstantMix")]
pub fn instant_mix(item_id: Uuid, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {    let catalog = state.catalog.read().unwrap();
    let seed = catalog.item(item_id).map(|item| item.name().to_string());
    let mut tracks = catalog.random_tracks(25);
    // Mixes seeded by the source item keep a stable flavor per item.
    if let Some(seed) = seed {
        tracks.sort_by_key(|track| format!("{}:{}", seed, track.id()));
    }
    let dtos = tracks.iter().map(|track| base_item(&catalog, track, None)).collect();
    Json(QueryResult { items: dtos, total_record_count: tracks.len(), start_index: 0 })
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
