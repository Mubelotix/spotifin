use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde_json::Value;
use uuid::Uuid;

use crate::AppState;

fn time_code(ms: u64) -> String {
    format!("{:02}:{:02}:{:02}.{:03}", ms / 3_600_000, ms % 3_600_000 / 60_000, ms % 60_000 / 1000, ms % 1000)
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

    let lines = crate::spotify::lyrics(&state.bridge, &uri).await.map_err(|_| Status::NotFound)?;
    if lines.is_empty() {
        return Err(Status::NotFound);
    }

    let synced = lines.iter().any(|line| line.start_ms > 0);
    let lyrics: Vec<Value> = lines
        .iter()
        .map(|line| {
            serde_json::json!({
                "Text": line.text,
                "Start": time_code(line.start_ms),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "Metadata": {
            "Title": title,
            "Artist": artist,
            "IsSynced": synced,
        },
        "Lyrics": lyrics,
    })))
}
