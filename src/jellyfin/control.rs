use std::sync::{Arc, Mutex};

use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, FromForm, State};
use rocket_ws::{Channel, Message, WebSocket};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::AppState;

#[derive(Clone, Default)]
pub struct ControlState {
    sender: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
}

impl ControlState {
    fn send(&self, message: Value) -> Result<(), Status> {
        let sender = self.sender.lock().unwrap().clone().ok_or(Status::NotFound)?;
        sender
            .send(Message::text(message.to_string()))
            .map_err(|_| Status::ServiceUnavailable)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Capabilities {
    #[serde(default)]
    pub playable_media_types: Vec<String>,
    #[serde(default)]
    pub supported_commands: Vec<String>,
    #[serde(default)]
    pub supports_media_control: bool,
    #[serde(default = "default_persistent_identifier")]
    pub supports_persistent_identifier: bool,
}

fn default_persistent_identifier() -> bool {
    true
}

#[derive(FromForm)]
pub struct SocketQuery<'r> {
    #[field(name = "deviceid")]
    device_id: Option<&'r str>,
    #[field(name = "apikey")]
    api_key: Option<&'r str>,
}

/// Registers the client capabilities expected by Jellyfin clients before they
/// open the session control socket. The session is intentionally single-user.
#[post("/Sessions/Capabilities/Full", data = "<body>")]
pub fn capabilities(state: &State<AppState>, body: Json<Capabilities>) -> Status {
    eprintln!(
        "jellyfin capabilities: media={:?} commands={:?} media_control={} persistent_id={}",
        body.playable_media_types,
        body.supported_commands,
        body.supports_media_control,
        body.supports_persistent_identifier,
    );
    let _ = state;
    Status::NoContent
}

/// Returns the one Spotify-backed player session exposed by this server.
#[get("/Sessions")]
pub fn sessions(state: &State<AppState>) -> Json<Vec<Value>> {
    let connected = state.control.sender.lock().unwrap().is_some();
    let item = serde_json::json!({
        "Id": "spotify-mcp",
        "UserId": crate::jellyfin::auth::user_id(),
        "DeviceId": "spotify-mcp",
        "DeviceName": "Spotify",
        "Client": "spotify-mcp",
        "ApplicationVersion": env!("CARGO_PKG_VERSION"),
        "IsActive": connected,
        "SupportsMediaControl": true,
        "SupportedCommands": ["Play", "Pause", "PlayPause", "Stop", "NextTrack", "PreviousTrack", "Seek"],
        "NowPlayingItem": Value::Null,
    });
    Json(vec![item])
}

/// Sends a standard Jellyfin playstate command to the connected client.
#[post("/Sessions/<session_id>/Playing/<command>?<seek_position_ticks>")]
pub async fn playstate(
    state: &State<AppState>,
    session_id: &str,
    command: &str,
    seek_position_ticks: Option<i64>,
) -> Status {
    if session_id != "spotify-mcp" {
        return Status::NotFound;
    }
    let mut data = serde_json::json!({ "Command": command });
    if let Some(ticks) = seek_position_ticks {
        data["SeekPositionTicks"] = ticks.into();
    }
    eprintln!("jellyfin control command: {command}");
    match state.control.send(serde_json::json!({
        "MessageType": "Playstate",
        "Data": data,
    })) {
        Ok(()) => Status::NoContent,
        Err(status) => {
            eprintln!("jellyfin control command failed: {command} ({status})");
            status
        }
    }
}

/// Jellyfin's session control socket. The access token is accepted for the
/// same reason the rest of this server accepts the static Jellyfin token, but
/// is never logged.
#[get("/socket?<query..>")]
pub fn socket(
    ws: WebSocket,
    state: &State<AppState>,
    query: SocketQuery<'_>,
) -> Channel<'static> {
    let control = state.control.clone();
    let device = query.device_id.unwrap_or("unknown").to_string();
    let authenticated = query.api_key == Some(crate::jellyfin::auth::ACCESS_TOKEN);
    ws.channel(move |stream| {
        Box::pin(async move {
            if !authenticated {
                eprintln!("jellyfin socket rejected device={device}: invalid token");
                return Err(rocket_ws::result::Error::ConnectionClosed);
            }
            eprintln!("jellyfin socket connected device={device}");
            let (mut sink, mut source) = stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            *control.sender.lock().unwrap() = Some(tx.clone());

            // Official Jellyfin servers use this message to negotiate the
            // keepalive interval with clients such as Linthra.
            sink.send(Message::text(
                serde_json::json!({
                    "MessageType": "ForceKeepAlive",
                    "Data": 60,
                })
                .to_string(),
            ))
            .await?;
            eprintln!("jellyfin socket sent ForceKeepAlive device={device}");

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
                        Some(Ok(Message::Text(text))) => {
                            log_client_message(&device, &text);
                        }
                        Some(Ok(_)) => {}
                        Some(Err(error)) => break Err(error),
                        None => break Ok(()),
                    },
                }
            };

            let mut sender = control.sender.lock().unwrap();
            if sender.as_ref().is_some_and(|current| current.same_channel(&tx)) {
                *sender = None;
            }
            eprintln!("jellyfin socket disconnected device={device}");
            outcome
        })
    })
}

fn log_client_message(device: &str, text: &str) {
    match serde_json::from_str::<Value>(text) {
        Ok(message) => {
            let kind = message
                .get("MessageType")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            eprintln!("jellyfin socket message device={device} type={kind}");
        }
        Err(_) => eprintln!("jellyfin socket invalid message device={device}"),
    }
}
