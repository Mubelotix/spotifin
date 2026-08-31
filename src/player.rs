use std::process::Command;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::bridge::{eval_on_bridge, BridgeState};

/// Serializes playback switches so concurrent audio requests (clients open
/// several connections per track) do not fight over the recorder.
#[derive(Default)]
pub struct PlayerControl {
    last_item: Mutex<Option<uuid::Uuid>>,
    session: Mutex<Option<CaptureSession>>,
}

/// Bounds an audio response to the requested track's lifetime: the stream
/// ends once the song is over instead of running like a radio.
#[derive(Clone, Copy)]
pub struct CaptureSession {
    pub item: uuid::Uuid,
    /// Recording less than the full song is never useful.
    pub min_end: Instant,
    /// The recorder runs at a constant 192 kbps, so the track's length in
    /// bytes is exact.
    pub expected_bytes: u64,
}

impl PlayerControl {
    /// Capture plan for `item`, if it is the one being recorded.
    pub async fn session_for(&self, item: uuid::Uuid) -> Option<CaptureSession> {
        match self.session.lock().await.as_ref() {
            Some(session) if session.item == item => Some(*session),
            _ => None,
        }
    }
}

const PLAY_JS: &str = r#"
(async () => {
    const uri = URI_PLACEHOLDER;
    const parts = uri.split(":");
    if (parts[1] === "track") {
        try { Spicetify.Platform.History.push("/track/" + parts[2]); } catch (e) {}
        await new Promise(r => setTimeout(r, 1200));
    }
    await Spicetify.Player.playUri(uri);
    await new Promise(r => setTimeout(r, 2500));
    const state = await Spicetify.Platform.PlayerAPI.getState();
    const current = state?.item?.uri ?? null;
    if (current !== uri) throw new Error("playing " + current);
    return state?.item?.name ?? "ok";
})()
"#;

/// Navigates to the track page and starts playback, verifying the renderer
/// actually switched. Returns the now-playing title.
async fn play_uri(bridge: &BridgeState, uri: &str) -> Result<String, String> {
    let literal = serde_json::to_string(uri).map_err(|e| e.to_string())?;
    let code = PLAY_JS.replace("URI_PLACEHOLDER", &literal);
    match eval_on_bridge(bridge, code).await {
        Ok(response) => Ok(response
            .get("value")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown track")
            .to_string()),
        Err(error) => Err(error.to_string()),
    }
}

fn reset_recorder() {
    // Kill only the ffmpeg writing the raw recording (anchored so the
    // supervisor shell whose cmdline mentions the same file survives); the
    // container respawns it immediately with a truncated output file.
    let _ = Command::new("pkill").args(["-9", "-f", "^ffmpeg.*recording\\.aac"]).status();
}

async fn wait_recorder_restarted(path: &std::path::Path, previous_len: u64) {
    for _ in 0..80 {
        if let Ok(meta) = tokio::fs::metadata(path).await {
            if meta.len() < previous_len {
                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// Makes the shared recording correspond to `item_id`: plays the track in the
/// client, resets the recorder and opens a capture session so audio responses
/// end with the song. Unknown items stream whatever is playing.
pub async fn prepare(state: &crate::AppState, item_id: uuid::Uuid, recording: &std::path::Path) {
    let (uri, duration_ms) = {
        let catalog = state.catalog.read().unwrap();
        match catalog.tracks.get(&item_id) {
            Some(track) => (Some(track.uri.clone()), track.duration_ms),
            None => (None, 0),
        }
    };
    let Some(uri) = uri else {
        return;
    };

    let mut last = state.player.last_item.lock().await;
    if *last == Some(item_id) {
        return;
    }

    let previous_len = tokio::fs::metadata(recording).await.map(|m| m.len()).unwrap_or(0);
    match play_uri(&state.bridge, &uri).await {
        Ok(title) => {
            eprintln!("now playing: {title}");
            reset_recorder();
            wait_recorder_restarted(recording, previous_len).await;
            let min_end = Instant::now() + Duration::from_millis(duration_ms);
            let expected_bytes = duration_ms / 1000 * 24_000;
            *state.player.session.lock().await =
                Some(CaptureSession { item: item_id, min_end, expected_bytes });
            *last = Some(item_id);
        }
        Err(error) => eprintln!("could not play {uri}: {error}"),
    }
}
