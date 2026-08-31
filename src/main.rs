use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use rocket::{
    data::{Data, ByteUnit},
    fs::NamedFile,
    get,
    http::{ContentType, Status},
    post,
    response::{Responder, Response},
    routes, State,
};
use rocket::futures::{SinkExt, StreamExt};
use rocket_ws::{Channel, Message, WebSocket};
use serde_json::Value;
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};

const EVAL_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Default)]
struct BridgeState {
    sender: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Value>>>,
    next_id: AtomicU64,
}

#[derive(Clone)]
struct AppState {
    recording: Arc<PathBuf>,
    hls: Arc<PathBuf>,
    bridge: Arc<BridgeState>,
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

fn resolve_result(bridge: &BridgeState, text: &str) {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return;
    };
    if value.get("type").and_then(Value::as_str) != Some("result") {
        return;
    }
    let Some(id) = value.get("id").and_then(Value::as_u64) else {
        return;
    };
    if let Some(waiter) = bridge.pending.lock().unwrap().remove(&id) {
        let _ = waiter.send(value);
    }
}

#[get("/ws")]
fn ws(ws: WebSocket, state: &State<AppState>) -> Channel<'static> {
    let bridge = state.bridge.clone();
    ws.channel(move |stream| {
        Box::pin(async move {
            let (mut sink, mut source) = stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            *bridge.sender.lock().unwrap() = Some(tx);

            let outcome = loop {
                tokio::select! {
                    outgoing = rx.recv() => match outgoing {
                        Some(message) => {
                            if sink.send(message).await.is_err() {
                                break Err(rocket_ws::result::Error::ConnectionClosed);
                            }
                        }
                        None => break Ok(()),
                    },
                    incoming = source.next() => match incoming {
                        Some(Ok(Message::Text(text))) => resolve_result(&bridge, &text),
                        Some(Ok(_)) => {}
                        Some(Err(error)) => break Err(error),
                        None => break Ok(()),
                    },
                }
            };

            *bridge.sender.lock().unwrap() = None;
            outcome
        })
    })
}

const NOT_CONNECTED: &str = "{\"error\":\"spicetify bridge not connected\"}";
const FORBIDDEN: &str = "{\"error\":\"debug eval is disabled\"}";

fn eval_enabled() -> bool {
    matches!(
        env::var("DEBUG_EVAL").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

async fn eval_on_bridge(
    bridge: &BridgeState,
    code: String,
) -> Result<(ContentType, String), (Status, String)> {
    let Some(sender) = bridge.sender.lock().unwrap().clone() else {
        return Err((Status::ServiceUnavailable, NOT_CONNECTED.into()));
    };

    let id = bridge.next_id.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    bridge.pending.lock().unwrap().insert(id, tx);

    let request = serde_json::json!({ "type": "eval", "id": id, "code": code });
    if sender.send(Message::text(request.to_string())).is_err() {
        bridge.pending.lock().unwrap().remove(&id);
        return Err((Status::ServiceUnavailable, NOT_CONNECTED.into()));
    }

    match timeout(EVAL_TIMEOUT, rx).await {
        Ok(Ok(response)) if response.get("ok").and_then(Value::as_bool) == Some(true) => {
            Ok((ContentType::JSON, response.to_string()))
        }
        Ok(Ok(response)) => {
            let error = response
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            Err((Status::BadRequest, serde_json::json!({ "error": error }).to_string()))
        }
        Ok(Err(_)) => Err((
            Status::BadGateway,
            "{\"error\":\"bridge dropped the request\"}".into(),
        )),
        Err(_) => {
            bridge.pending.lock().unwrap().remove(&id);
            Err((
                Status::GatewayTimeout,
                "{\"error\":\"extension did not answer in time\"}".into(),
            ))
        }
    }
}

#[post("/debug/eval", data = "<body>")]
async fn debug_eval(
    state: &State<AppState>,
    body: Data<'_>,
) -> Result<(ContentType, String), (Status, String)> {
    if !eval_enabled() {
        return Err((Status::Forbidden, FORBIDDEN.into()));
    }
    let bytes = match body.open(ByteUnit::MiB).into_bytes().await {
        Ok(capped) => capped.value,
        Err(_) => Vec::new(),
    };
    let code = String::from_utf8_lossy(&bytes).into_owned();
    eval_on_bridge(&state.bridge, code).await
}

#[rocket::launch]
async fn rocket() -> _ {
    let root = data_dir();
    let state = AppState {
        recording: Arc::new(root.join("recording.aac")),
        hls: Arc::new(root.join("hls")),
        bridge: Arc::new(BridgeState::default()),
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
            ws,
            debug_eval
        ],
    )
}
