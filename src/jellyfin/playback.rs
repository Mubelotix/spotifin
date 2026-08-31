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

/// Playback reporting is stored as-is for UserData; the Spotify client owns
/// the real cursor, so these values are informational only.
#[post("/Sessions/Playing", data = "<body>")]
pub async fn playing_started(state: &State<AppState>, body: Json<Value>) {
    if let Some(item) = body.get("ItemId").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok()) {
        state.player.note_requested(item).await;
    }
    record(state, &body, Report::Start);
}

#[post("/Sessions/Playing/Progress", data = "<body>")]
pub fn playing_progress(state: &State<AppState>, body: Json<Value>) {
    record(state, &body, Report::Progress);
}

#[post("/Sessions/Playing/Stopped", data = "<body>")]
pub fn playing_stopped(state: &State<AppState>, body: Json<Value>) {
    record(state, &body, Report::Stop);
}

enum Report {
    Start,
    Progress,
    Stop,
}

fn record(state: &State<AppState>, body: &Value, kind: Report) {
    let Some(item) = body.get("ItemId").and_then(Value::as_str).and_then(|s| Uuid::parse_str(s).ok()) else {
        return;
    };
    let ticks = body.get("PositionTicks").and_then(Value::as_u64).unwrap_or(0);
    let mut catalog = state.catalog.write().unwrap();
    match kind {
        Report::Start => catalog.note_started(item),
        Report::Progress => catalog.note_progress(item, ticks),
        Report::Stop => catalog.note_stopped(item, ticks),
    }
}

#[get("/Sessions/Playing/Ping")]
pub fn playing_ping() -> Status {
    rocket::http::Status::Ok
}
