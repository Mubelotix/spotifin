use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rocket::fs::NamedFile;
use rocket::http::{ContentType, Status};
use rocket::request::Outcome;
use rocket::response::{Responder, Response};
use rocket::{get, routes, Route, State, Request};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    time::sleep,
};

use crate::player;

pub fn routes() -> Vec<Route> {
    routes![universal, file_alias, stream_alias, playlist, segment, relative_segment]
}

pub struct AudioResponse(Response<'static>);

impl<'r> Responder<'r, 'static> for AudioResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(self.0)
    }
}

/// Streams the recording from its live tail, restarting at offset zero when
/// the recorder was reset (the file shrinks). A byte range bounds the stream
/// to the requested window.
fn audio_stream(
    path: Arc<PathBuf>,
    session: Option<player::CaptureSession>,
    window: Option<(u64, u64, u64)>,
    complete_file: bool,
    partial: bool,
) -> AudioResponse {
    let (reader, mut writer) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let mut offset = window.map(|(start, _, _)| start).unwrap_or(0);
        loop {
            if let Some((_, end, _)) = window {
                if offset > end {
                    break;
                }
            }

            let Ok(mut file) = tokio::fs::File::open(&*path).await else {
                sleep(Duration::from_millis(250)).await;
                continue;
            };
            if file.seek(std::io::SeekFrom::Start(offset)).await.is_err() {
                sleep(Duration::from_millis(250)).await;
                continue;
            }

            if let Some(capture) = session {
                let now = std::time::Instant::now();
                if offset >= capture.expected_bytes && now >= capture.min_end {
                    break;
                }
            }

            let mut buffer = vec![0; 32 * 1024];
            if let Some((_, end, _)) = window {
                let remaining = (end + 1 - offset.min(end + 1)) as usize;
                if remaining == 0 {
                    break;
                }
                buffer.truncate(remaining);
            }
            match file.read(&mut buffer).await {
                Ok(0) => {
                    if complete_file {
                        break;
                    }
                    // Nothing new; if the file shrank the recorder was reset.
                    if let Ok(meta) = file.metadata().await {
                        if meta.len() < offset {
                            offset = 0;
                        }
                    }
                    sleep(Duration::from_millis(250)).await
                }
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

    let mut builder = Response::build();
    builder.header(ContentType::AAC);
    builder.raw_header("Cache-Control", "no-store");
    builder.raw_header("Accept-Ranges", "bytes");
    if partial {
        builder.status(Status::PartialContent);
        if let Some((start, end, total)) = window {
            builder.raw_header("Content-Range", format!("bytes {start}-{end}/{total}"));
        }
    }
    AudioResponse(builder.streamed_body(reader).finalize())
}
/// Parses single-range `bytes=a-b`, `bytes=a-` and `bytes=-b` headers against
/// the canonical size `total`.
fn parse_range(header: &str, total: u64) -> Option<(u64, u64)> {
    let first = header.strip_prefix("bytes=")?.split(',').next()?.trim();
    let (start_s, end_s) = first.split_once('-')?;
    match (start_s.trim().parse::<u64>(), end_s.trim().parse::<u64>()) {
        (Ok(start), Ok(end)) if start <= end && start < total => Some((start, end.min(total - 1))),
        (Ok(start), Err(_)) if start < total => Some((start, total - 1)),
        (Err(_), Ok(length)) if length > 0 => {
            let length = length.min(total);
            Some((total - length, total - 1))
        }
        _ => None,
    }
}

/// Captures the raw `Range` header, if any.
pub struct RangeHeader(Option<String>);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for RangeHeader {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let header = request.headers().get_one("Range").map(str::to_string);
        Outcome::Success(RangeHeader(header))
    }
}

/// A cached capture qualifies when its tail is at most a few seconds short of
/// the track's expected size.
async fn cached_capture(state: &State<crate::AppState>, id: uuid::Uuid) -> Option<(PathBuf, u64)> {
    let duration_ms = state.catalog.read().unwrap().tracks.get(&id)?.duration_ms;
    if duration_ms == 0 {
        return None;
    }
    let expected_bytes = duration_ms / 1000 * 24_000;
    let path = state.audio.cache.join(format!("{id}.aac"));
    let len = tokio::fs::metadata(&path).await.ok()?.len();
    (len + 6 * 24_000 >= expected_bytes).then_some((path, len))
}

async fn open_audio_stream(
    state: &State<crate::AppState>,
    item_id: &str,
    range: RangeHeader,
) -> Result<AudioResponse, rocket::http::Status> {
    let Ok(id) = uuid::Uuid::parse_str(item_id) else {
        return Err(rocket::http::Status::NotFound);
    };
    if !state.catalog.read().unwrap().tracks.contains_key(&id) {
        return Err(rocket::http::Status::NotFound);
    }
    let explicit_range = range.0.is_some();
    if let Some((path, len)) = cached_capture(state, id).await {
        // Complete file on disk: no client interaction, serve the whole thing
        // or any requested slice of it.
        state.player.note_activity();
        let window = range
            .0
            .and_then(|header| parse_range(&header, len))
            .map(|(start, end)| (start, end, len))
            .or(Some((0, len.saturating_sub(1), len)));
        return Ok(audio_stream(
            Arc::new(path),
            None,
            window,
            true,
            explicit_range,
        ));
    }
    player::prepare(state, id, &state.audio.recording, &state.audio.cache).await;

    let session = state.player.session_for(id).await;
    let total = match &session {
        Some(capture) => capture.expected_bytes,
        None => tokio::fs::metadata(state.audio.recording.as_ref()).await.map(|m| m.len()).unwrap_or(0),
    };
    let explicit_range = range.0.is_some();
    let window = range
        .0
        .and_then(|header| parse_range(&header, total))
        .map(|(start, end)| (start, end, total));
    Ok(audio_stream(
        state.audio.recording.clone(),
        session,
        window,
        false,
        explicit_range,
    ))
}

/// All audio items stream the shared live recording; requesting a known item
/// switches the client to that track first.
#[get("/Audio/<item_id>/universal")]
pub async fn universal(state: &State<crate::AppState>, item_id: &str, range: RangeHeader) -> Result<AudioResponse, rocket::http::Status> {
    open_audio_stream(state, item_id, range).await
}

#[get("/Audio/<item_id>/File")]
pub async fn file_alias(state: &State<crate::AppState>, item_id: &str, range: RangeHeader) -> Result<AudioResponse, rocket::http::Status> {
    open_audio_stream(state, item_id, range).await
}

#[get("/Audio/<item_id>/stream")]
pub async fn stream_alias(state: &State<crate::AppState>, item_id: &str, range: RangeHeader) -> Result<AudioResponse, rocket::http::Status> {
    open_audio_stream(state, item_id, range).await
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
