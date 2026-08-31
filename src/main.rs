use std::{env, path::PathBuf, sync::Arc, time::Duration};

use rocket::{
    fs::NamedFile,
    get,
    http::{ContentType, Status},
    response::{Responder, Response},
    routes, State,
};
use rocket_ws::{Message, Stream, WebSocket};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    time::sleep,
};

#[derive(Clone)]
struct AppState {
    recording: Arc<PathBuf>,
    hls: Arc<PathBuf>,
}

fn data_dir() -> PathBuf {
    env::var_os("AUDIO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/audio"))
}

fn valid_component(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && !value.contains('/')
}

async fn ensure_dirs(state: &AppState) -> std::io::Result<()> {
    tokio::fs::create_dir_all(&*state.hls).await?;
    if let Some(parent) = state.recording.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    Ok(())
}

struct AudioResponse(Response<'static>);

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

#[get("/Audio/<item_id>/universal")]
fn universal(state: &State<AppState>, item_id: &str) -> AudioResponse {
    let _ = item_id;
    audio_stream(state.recording.clone())
}

#[get("/Items/<item_id>/File")]
fn file_alias(state: &State<AppState>, item_id: &str) -> AudioResponse {
    let _ = item_id;
    audio_stream(state.recording.clone())
}

#[get("/Audio/<item_id>/main.m3u8")]
async fn playlist(state: &State<AppState>, item_id: &str) -> Option<(ContentType, NamedFile)> {
    let _ = item_id;
    NamedFile::open(state.hls.join("main.m3u8"))
        .await
        .ok()
        .map(|file| (ContentType::new("application", "vnd.apple.mpegurl"), file))
}

#[get("/Audio/<item_id>/hls1/<playlist_id>/<segment>")]
async fn segment(
    state: &State<AppState>,
    item_id: &str,
    playlist_id: &str,
    segment: &str,
) -> Option<(ContentType, NamedFile)> {
    let _ = (item_id, playlist_id);
    if !valid_component(segment) {
        return None;
    }
    NamedFile::open(state.hls.join(segment))
        .await
        .ok()
        .map(|file| (ContentType::new("video", "mp2t"), file))
}

#[get("/Audio/<item_id>/<segment>", rank = 2)]
async fn relative_segment(
    state: &State<AppState>,
    item_id: &str,
    segment: &str,
) -> Option<(ContentType, NamedFile)> {
    let _ = item_id;
    if !valid_component(segment) || !segment.ends_with(".ts") {
        return None;
    }
    NamedFile::open(state.hls.join(segment))
        .await
        .ok()
        .map(|file| (ContentType::new("video", "mp2t"), file))
}

#[get("/health")]
fn health() -> Status {
    Status::Ok
}

#[get("/ws")]
fn ws(ws: WebSocket) -> Stream![] {
    Stream! { ws =>
        for await message in ws {
            match message? {
                Message::Text(text) => yield Message::Text(format!("ack:{text}")),
                Message::Ping(payload) => yield Message::Pong(payload),
                Message::Close(_) => break,
                _ => {}
            }
        }
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    let root = data_dir();
    let state = AppState {
        recording: Arc::new(root.join("recording.aac")),
        hls: Arc::new(root.join("hls")),
    };

    if let Err(error) = ensure_dirs(&state).await {
        eprintln!("could not create audio directories: {error}");
    }

    rocket::build().manage(state).mount(
        "/",
        routes![
            universal,
            file_alias,
            playlist,
            segment,
            relative_segment,
            health,
            ws
        ],
    )
}
