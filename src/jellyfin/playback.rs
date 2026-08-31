use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use serde_json::Value;
use uuid::Uuid;

use crate::AppState;

/// Playback info: the audio itself always streams from the shared recording,
/// so a single direct-play source is reported for every track.
#[get("/Items/<item_id>/PlaybackInfo")]
pub fn playback_info_get(item_id: Uuid, state: &State<AppState>) -> Result<Json<Value>, Status> {
    playback_info(item_id, state)
}

#[post("/Items/<item_id>/PlaybackInfo", data = "<_profile>")]
pub fn playback_info_post(item_id: Uuid, _profile: Json<Value>, state: &State<AppState>) -> Result<Json<Value>, Status> {
    playback_info(item_id, state)
}

fn playback_info(item_id: Uuid, state: &State<AppState>) -> Result<Json<Value>, Status> {
    let catalog = state.catalog.read().unwrap();
    let duration_ticks = match catalog.item(item_id) {
        Some(crate::catalog::Item::Track(track)) => track.duration_ms * 10_000,
        _ => return Err(Status::NotFound),
    };
    Ok(Json(serde_json::json!({
        "MediaSources": [{
            "Id": item_id,
            "Path": format!("/Audio/{item_id}/stream"),
            "Protocol": "Http",
            "Container": "aac",
            "RunTimeTicks": duration_ticks,
            "SupportsDirectPlay": true,
            "SupportsDirectStream": true,
            "SupportsTranscoding": true,
            "MediaStreams": [{
                "Type": "Audio",
                "Index": 0,
                "Codec": "aac",
                "Channels": 2,
                "SampleRate": 44100,
                "BitRate": 192000,
                "IsDefault": true
            }]
        }],
        "PlaySessionId": item_id.to_string(),
        "ErrorCode": null
    })))
}

/// Playback reporting is accepted but not persisted; the Spotify client owns
/// the playback cursor.
#[post("/Sessions/Playing", data = "<_body>")]
pub fn playing_started(_body: Json<Value>) {}

#[post("/Sessions/Playing/Progress", data = "<_body>")]
pub fn playing_progress(_body: Json<Value>) {}

#[post("/Sessions/Playing/Stopped", data = "<_body>")]
pub fn playing_stopped(_body: Json<Value>) {}

#[get("/Sessions/Playing/Ping")]
pub fn playing_ping() -> Status {
    rocket::http::Status::Ok
}
