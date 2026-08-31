use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::response::{Responder, Response};
use rocket::{get, routes, Route, State};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    time::sleep,
};

pub fn routes() -> Vec<Route> {
    routes![universal, file_alias, stream_alias, playlist, segment, relative_segment]
}

pub struct AudioResponse(Response<'static>);

impl<'r> Responder<'r, 'static> for AudioResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(self.0)
    }
}

fn audio_stream(path: Arc<PathBuf>) -> AudioResponse {
    let (reader, mut writer) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let mut offset = 0u64;
        loop {
            let Ok(mut file) = tokio::fs::File::open(&*path).await else {
                sleep(Duration::from_millis(250)).await;
                continue;
            };
            if file.seek(std::io::SeekFrom::Start(offset)).await.is_err() {
                sleep(Duration::from_millis(250)).await;
                continue;
            }

            let mut buffer = vec![0; 32 * 1024];
            match file.read(&mut buffer).await {
                Ok(0) => sleep(Duration::from_millis(250)).await,
                Ok(size) => {
                    buffer.truncate(size);
                    offset += size as u64;
                    if writer.write_all(&buffer).await.is_err() {
                        break;
                    }
                }
                Err(_) => sleep(Duration::from_millis(250)).await,
            }
        }
    });

    AudioResponse(
        Response::build()
            .header(ContentType::AAC)
            .raw_header("Cache-Control", "no-store")
            .streamed_body(reader)
            .finalize(),
    )
}

/// All audio items stream the same live recording for now; the Jellyfin
/// playback cursor is not managed.
#[get("/Audio/<item_id>/universal")]
pub fn universal(state: &State<crate::AppState>, item_id: &str) -> AudioResponse {
    let _ = item_id;
    audio_stream(state.audio.recording.clone())
}

#[get("/Audio/<item_id>/File")]
pub fn file_alias(state: &State<crate::AppState>, item_id: &str) -> AudioResponse {
    let _ = item_id;
    audio_stream(state.audio.recording.clone())
}

#[get("/Audio/<item_id>/stream")]
pub fn stream_alias(state: &State<crate::AppState>, item_id: &str) -> AudioResponse {
    let _ = item_id;
    audio_stream(state.audio.recording.clone())
}

#[get("/Audio/<item_id>/main.m3u8")]
pub async fn playlist(state: &State<crate::AppState>, item_id: &str) -> Option<(ContentType, NamedFile)> {
    let _ = item_id;
    NamedFile::open(state.audio.hls.join("main.m3u8"))
        .await
        .ok()
        .map(|file| (ContentType::new("application", "vnd.apple.mpegurl"), file))
}

#[get("/Audio/<item_id>/hls1/<playlist_id>/<segment>")]
pub async fn segment(
    state: &State<crate::AppState>,
    item_id: &str,
    playlist_id: &str,
    segment: &str,
) -> Option<(ContentType, NamedFile)> {
    let _ = (item_id, playlist_id);
    NamedFile::open(state.audio.hls.join(segment))
        .await
        .ok()
        .map(|file| (ContentType::new("video", "mp2t"), file))
}

#[get("/Audio/<item_id>/<segment>", rank = 2)]
pub async fn relative_segment(state: &State<crate::AppState>, item_id: &str, segment: &str) -> Option<(ContentType, NamedFile)> {
    let _ = item_id;
    if !segment.ends_with(".ts") || segment.contains('/') {
        return None;
    }
    NamedFile::open(state.audio.hls.join(segment))
        .await
        .ok()
        .map(|file| (ContentType::new("video", "mp2t"), file))
}
