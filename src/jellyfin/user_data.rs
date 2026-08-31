use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, State};
use uuid::Uuid;

use crate::jellyfin::dto::{user_data, UserItemData};
use crate::AppState;

#[get("/Users/<user_id>/Items/<item_id>/UserData")]
pub fn get_user_data(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Option<Json<UserItemData>> {
    check_user(user_id)?;
    let catalog = state.catalog.read().unwrap();
    catalog.item(item_id)?;
    Some(Json(user_data(&catalog, item_id)))
}

#[post("/Users/<user_id>/Items/<item_id>/UserData", data = "<body>")]
pub fn post_user_data(user_id: Uuid, item_id: Uuid, body: Json<serde_json::Value>, state: &State<AppState>) -> Json<UserItemData> {
    update_user_data(user_id, item_id, body, state)
}

fn update_user_data(user_id: Uuid, item_id: Uuid, body: Json<serde_json::Value>, state: &State<AppState>) -> Json<UserItemData> {
    if check_user(user_id).is_none() {
        return Json(UserItemData {
            item_id,
            is_favorite: false,
            played: false,
            play_count: 0,
            playback_position_ticks: 0,
            last_played_date: None,
        });
    }
    if let Some(favorite) = body.0.get("IsFavorite").and_then(serde_json::Value::as_bool) {
        state.catalog.write().unwrap().set_favorite(item_id, favorite);
    }
    let catalog = state.catalog.read().unwrap();
    Json(user_data(&catalog, item_id))
}

#[post("/Users/<user_id>/FavoriteItems/<item_id>")]
pub fn mark_favorite(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    set_favorite(user_id, item_id, true, state)
}

#[delete("/Users/<user_id>/FavoriteItems/<item_id>")]
pub fn unmark_favorite(user_id: Uuid, item_id: Uuid, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    set_favorite(user_id, item_id, false, state)
}

fn set_favorite(user_id: Uuid, item_id: Uuid, favorite: bool, state: &State<AppState>) -> Result<Json<UserItemData>, Status> {
    check_user(user_id).ok_or(Status::NotFound)?;
    {
        let mut catalog = state.catalog.write().unwrap();
        catalog.item(item_id).ok_or(Status::NotFound)?;
        catalog.set_favorite(item_id, favorite);
    }
    let catalog = state.catalog.read().unwrap();
    Ok(Json(user_data(&catalog, item_id)))
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
