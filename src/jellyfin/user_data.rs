use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};
use uuid::Uuid;

use crate::jellyfin::dto::{user_data, UserItemData};
use crate::AppState;

fn empty_user_data(item_id: Uuid) -> Json<UserItemData> {
    Json(UserItemData {
        item_id,
        key: item_id.to_string(),
        is_favorite: false,
        likes: None,
        played: false,
        play_count: 0,
        playback_position_ticks: 0,
        last_played_date: None,
    })
}

#[get("/Users/<user_id>/Items/<item_id>/UserData")]
pub fn get_user_data(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Option<Json<UserItemData>> {
    check_user(user_id)?;
    let catalog = state.catalog.read().unwrap();
    catalog.item(item_id)?;
    Some(Json(user_data(&catalog, item_id)))
}

#[post("/Users/<user_id>/Items/<item_id>/UserData", data = "<body>")]
pub async fn post_user_data(user_id: Uuid, item_id: Uuid, body: Json<serde_json::Value>, state: &State<AppState>) -> Json<UserItemData> {
    update_user_data(user_id, item_id, body, state).await
}

async fn update_user_data(user_id: Uuid, item_id: Uuid, body: Json<serde_json::Value>, state: &State<AppState>) -> Json<UserItemData> {
    if check_user(user_id).is_none() {
        return empty_user_data(item_id);
    }
    if let Some(favorite) = body.0.get("IsFavorite").and_then(serde_json::Value::as_bool) {
        if set_spotify_favorite(state, item_id, favorite).await.is_ok() {
            state.catalog.write().unwrap().set_favorite(item_id, favorite);
        }
    }
    if let Some(likes) = body.0.get("Likes").and_then(serde_json::Value::as_bool) {
        state.catalog.write().unwrap().set_likes(item_id, Some(likes));
    }
    let catalog = state.catalog.read().unwrap();
    Json(user_data(&catalog, item_id))
}

#[post("/Users/<user_id>/FavoriteItems/<item_id>")]
pub async fn mark_favorite(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    set_favorite(user_id, item_id, true, state).await
}

#[delete("/Users/<user_id>/FavoriteItems/<item_id>")]
pub async fn unmark_favorite(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    set_favorite(user_id, item_id, false, state).await
}

#[post("/UserFavoriteItems/<item_id>")]
pub async fn mark_favorite_modern(item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    mark_favorite(crate::jellyfin::auth::user_id(), item_id, state).await
}

#[delete("/UserFavoriteItems/<item_id>")]
pub async fn unmark_favorite_modern(item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    unmark_favorite(crate::jellyfin::auth::user_id(), item_id, state).await
}

async fn set_favorite(user_id: Uuid, item_id: Uuid, favorite: bool, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    check_user(user_id).ok_or(Status::NotFound)?;
    let uri = {
        let catalog = state.catalog.read().unwrap();
        match catalog.item(item_id) {
            Some(crate::catalog::Item::Track(track)) => track.uri.clone(),
            Some(crate::catalog::Item::Album(album)) => album.uri.clone(),
            Some(crate::catalog::Item::Artist(artist)) => artist.uri.clone(),
            Some(_) => return Err(Status::BadRequest),
            None => return Err(Status::NotFound),
        }
    };
    crate::spotify::set_favorite(&state.bridge, &uri, favorite).await.map_err(|_| Status::BadGateway)?;
    state.catalog.write().unwrap().set_favorite(item_id, favorite);
    let catalog = state.catalog.read().unwrap();
    Ok(Json(user_data(&catalog, item_id)))
}

async fn set_spotify_favorite(state: &State<AppState>, item_id: Uuid, favorite: bool) -> Result<(), String> {
    let uri = {
        let catalog = state.catalog.read().unwrap();
        match catalog.item(item_id) {
            Some(crate::catalog::Item::Track(track)) => track.uri.clone(),
            Some(crate::catalog::Item::Album(album)) => album.uri.clone(),
            Some(crate::catalog::Item::Artist(artist)) => artist.uri.clone(),
            _ => return Err("item is not a track".into()),
        }
    };
    crate::spotify::set_favorite(&state.bridge, &uri, favorite).await
}

#[post("/UserItems/<item_id>/Rating?<likes>")]
pub fn set_rating(item_id: Uuid, likes: Option<bool>, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    let likes = likes.ok_or(Status::BadRequest)?;
    let mut catalog = state.catalog.write().unwrap();
    catalog.item(item_id).ok_or(Status::NotFound)?;
    catalog.set_likes(item_id, Some(likes));
    Ok(Json(user_data(&catalog, item_id)))
}

#[post("/Users/<user_id>/Items/<item_id>/Rating?<likes>")]
pub fn set_rating_legacy(user_id: Uuid, item_id: Uuid, likes: Option<bool>, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    check_user(user_id).ok_or(Status::NotFound)?;
    set_rating(item_id, likes, state)
}

#[delete("/UserItems/<item_id>/Rating")]
pub fn clear_rating(item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    let mut catalog = state.catalog.write().unwrap();
    catalog.item(item_id).ok_or(Status::NotFound)?;
    catalog.set_likes(item_id, None);
    Ok(Json(user_data(&catalog, item_id)))
}

#[delete("/Users/<user_id>/Items/<item_id>/Rating")]
pub fn clear_rating_legacy(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    check_user(user_id).ok_or(Status::NotFound)?;
    clear_rating(item_id, state)
}

#[post("/Users/<user_id>/PlayedItems/<item_id>")]
pub fn mark_played(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Status {
    report(user_id, item_id, state)
}

#[delete("/Users/<user_id>/PlayedItems/<item_id>")]
pub fn unmark_played(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Status {
    report(user_id, item_id, state)
}

fn report(user_id: Uuid, _item_id: Uuid, state: &State<AppState>) -> Status {
    match check_user(user_id) {
        Some(()) => {
            let _ = state;
            Status::NoContent
        }
        None => Status::NotFound,
    }
}

fn check_user(user_id: Uuid) -> Option<()> {
    (user_id == crate::jellyfin::auth::user_id()).then_some(())
}

/// Modern user-context variants of the legacy /Users/{userId}/... routes.
#[get("/UserItems/<item_id>/UserData")]
pub fn get_user_data_modern(item_id: Uuid, state: &State<AppState>) -> Option<Json<UserItemData>> {
    get_user_data(crate::jellyfin::auth::user_id(), item_id, state)
}

#[post("/UserItems/<item_id>/UserData", data = "<body>")]
pub async fn post_user_data_modern(
    item_id: Uuid,
    body: Json<serde_json::Value>,
    state: &State<AppState>,
) -> Json<UserItemData> {
    post_user_data(crate::jellyfin::auth::user_id(), item_id, body, state).await
}
