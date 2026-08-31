use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use rocket::data::{ByteUnit, Data};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::{ContentType, Status};
use rocket::{get, post, State};
use rocket_ws::{Channel, Message, WebSocket};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

const EVAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug)]
pub enum EvalError {
    NotConnected,
    Dropped,
    Timeout,
    Rejected(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::NotConnected => write!(formatter, "spicetify bridge not connected"),
            EvalError::Dropped => write!(formatter, "bridge dropped the request"),
            EvalError::Timeout => write!(formatter, "extension did not answer in time"),
            EvalError::Rejected(reason) => write!(formatter, "{reason}"),
        }
    }
}

#[derive(Default)]
struct Inner {
    sender: Mutex<Option<mpsc::UnboundedSender<Message>>>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Value>>>,
    next_id: AtomicU64,
}

#[derive(Clone, Default)]
pub struct BridgeState {
    inner: Arc<Inner>,
}

pub async fn eval_on_bridge(bridge: &BridgeState, code: String) -> Result<Value, EvalError> {
    let sender = bridge.inner.sender.lock().unwrap().clone();
    let Some(sender) = sender else {
        return Err(EvalError::NotConnected);
    };
    let id = bridge.inner.next_id.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    bridge.inner.pending.lock().unwrap().insert(id, tx);

    let request = serde_json::json!({ "type": "eval", "id": id, "code": code });
    if sender.send(Message::text(request.to_string())).is_err() {
        bridge.inner.pending.lock().unwrap().remove(&id);
        return Err(EvalError::NotConnected);
    }

    match timeout(EVAL_TIMEOUT, rx).await {
        Ok(Ok(response)) if response.get("ok").and_then(Value::as_bool) == Some(true) => Ok(response),
        Ok(Ok(response)) => Err(EvalError::Rejected(
            response.get("error").and_then(Value::as_str).unwrap_or("unknown error").to_string(),
        )),
        Ok(Err(_)) => Err(EvalError::Dropped),
        Err(_) => {
            bridge.inner.pending.lock().unwrap().remove(&id);
            Err(EvalError::Timeout)
        }
    }
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
    if let Some(waiter) = bridge.inner.pending.lock().unwrap().remove(&id) {
        let _ = waiter.send(value);
    }
}

#[get("/ws")]
pub fn ws(ws: WebSocket, state: &State<crate::AppState>) -> Channel<'static> {
    let bridge = state.bridge.clone();
    ws.channel(move |stream| {
        Box::pin(async move {
            let (mut sink, mut source) = stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            *bridge.inner.sender.lock().unwrap() = Some(tx);

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

            *bridge.inner.sender.lock().unwrap() = None;
            outcome
        })
    })
}

fn eval_enabled() -> bool {
    matches!(
        std::env::var("DEBUG_EVAL").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn status_of(error: &EvalError) -> Status {
    match error {
        EvalError::Rejected(_) => Status::BadRequest,
        EvalError::Timeout => Status::GatewayTimeout,
        _ => Status::ServiceUnavailable,
    }
}

#[post("/debug/eval", data = "<body>")]
pub async fn debug_eval(state: &State<crate::AppState>, body: Data<'_>) -> Result<(ContentType, String), (Status, String)> {
    if !eval_enabled() {
        return Err((Status::Forbidden, "{\"error\":\"debug eval is disabled\"}".into()));
    }
    let bytes = match body.open(ByteUnit::MiB).into_bytes().await {
        Ok(capped) => capped.value,
        Err(_) => Vec::new(),
    };
    let code = String::from_utf8_lossy(&bytes).into_owned();
    match eval_on_bridge(&state.bridge, code).await {
        Ok(response) => Ok((ContentType::JSON, response.to_string())),
        Err(error) => Err((status_of(&error), format!("{{\"error\":\"{error}\"}}"))),
    }
}
