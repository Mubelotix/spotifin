use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::FromForm;
use rocket::{get, State};
use uuid::Uuid;

use crate::catalog::{self, Catalog};
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

fn run_query(catalog: &Catalog, query: ItemQuery) -> QueryResult<BaseItemDto> {
    let types = parse_types(query.include_item_types);
    let parent = query.parent_id.as_deref().and_then(|raw| Uuid::parse_str(raw).ok());
    let found = catalog.query(&types, parent, query.search_term.as_deref());
    let start = query.start_index.unwrap_or(0);
    let (items, total) = page(found, start, query.limit);
    let dtos = items.iter().map(|item| base_item(catalog, item, None)).collect();
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
pub fn user_items(user_id: Uuid, query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    if user_id != crate::jellyfin::auth::user_id() {
        return Json(QueryResult { items: vec![], total_record_count: 0, start_index: 0 });
    }
    let catalog = state.catalog.read().unwrap();
    Json(run_query(&catalog, query))
}

#[get("/Items?<query..>", rank = 2)]
pub fn items(query: ItemQuery, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    let catalog = state.catalog.read().unwrap();
    Json(run_query(&catalog, query))
}

type MaybeItem = Result<Json<BaseItemDto>, Status>;

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
pub fn instant_mix(item_id: Uuid, state: &State<AppState>) -> Json<QueryResult<BaseItemDto>> {
    let catalog = state.catalog.read().unwrap();
    let seed = catalog.item(item_id).map(|item| item.name().to_string());
    let mut tracks = catalog.random_tracks(25);
    // Mixes seeded by the source item keep a stable flavor per item.
    if let Some(seed) = seed {
        tracks.sort_by_key(|track| format!("{}:{}", seed, track.id()));
    }
    let dtos = tracks.iter().map(|track| base_item(&catalog, track, None)).collect();
    Json(QueryResult { items: dtos, total_record_count: tracks.len(), start_index: 0 })
}
