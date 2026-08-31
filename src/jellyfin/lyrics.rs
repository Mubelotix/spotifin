use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde_json::Value;
use uuid::Uuid;

use crate::AppState;

fn ticks(ms: u64) -> u64 {
    ms * 10_000
}

/// TARGET.md LyricDto shape; synced lines carry their start time.
#[get("/Audio/<item_id>/Lyrics")]
pub async fn lyrics(item_id: Uuid, state: &State<AppState>) -> Result<Json<Value>, Status> {
    let (uri, title, artist) = {
        let catalog = state.catalog.read().unwrap();
        let track = catalog.tracks.get(&item_id).ok_or(Status::NotFound)?;
        let artist = track
            .artist_ids
            .first()
            .and_then(|id| catalog.artists.get(id))
            .map(|a| a.name.clone());
        (track.uri.clone(), track.name.clone(), artist)
    };

    let cache_path = crate::jellyfin::dto::lyrics_cache_path(item_id);
    if let Ok(raw) = tokio::fs::read(&cache_path).await {
        let cached: Value = serde_json::from_slice(&raw).map_err(|_| Status::NotFound)?;
        if let Some(lines) = cached.get("Lyrics").and_then(Value::as_array) {
            if !lines.is_empty() {
                return Ok(Json(cached));
            }
            return Err(Status::NotFound);
        }
    }

    let lines = crate::spotify::lyrics(&state.bridge, &uri).await.map_err(|_| Status::NotFound)?;
    let synced = lines.iter().any(|line| line.start_ms > 0);
    let lyrics: Vec<Value> = lines
        .iter()
        .map(|line| {
            serde_json::json!({
                "Text": line.text,
                "Start": ticks(line.start_ms),
                "Cues": line.cues.as_ref().map(|cues| cues.iter().map(|cue| serde_json::json!({
                    "Position": cue.position,
                    "EndPosition": cue.end_position,
                    "Start": ticks(cue.start_ms),
                    "End": cue.end_ms.map(ticks),
                })).collect::<Vec<_>>()),
            })
        })
        .collect();

    let response = serde_json::json!({
        "Metadata": {
            "Title": title,
            "Artist": artist,
            "IsSynced": synced,
        },
        "Lyrics": lyrics,
    });
    tokio::fs::create_dir_all(&*state.audio.cache).await.map_err(|_| Status::InternalServerError)?;
    tokio::fs::write(&cache_path, serde_json::to_vec(&response).map_err(|_| Status::InternalServerError)?)
        .await
        .map_err(|_| Status::InternalServerError)?;
    if lines.is_empty() {
        return Err(Status::NotFound);
    }
    Ok(Json(response))
}
