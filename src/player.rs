use std::process::Command;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
    Mutex as StdMutex,
};
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::bridge::{eval_on_bridge, BridgeState};

/// Serializes playback switches so concurrent audio requests (clients open
/// several connections per track) do not fight over the recorder.
#[derive(Default)]
struct PlayerInner {
    last_item: Mutex<Option<uuid::Uuid>>,
    switch_generation: AtomicU64,
    last_switch: Mutex<Option<Instant>>,
    requested_item: Mutex<Option<uuid::Uuid>>,
    session: Mutex<Option<CaptureSession>>,
    live_streams: AtomicUsize,
    last_activity: StdMutex<Option<Instant>>,
    idle_paused: StdMutex<bool>,
}

#[derive(Clone, Default)]
pub struct PlayerControl {
    inner: Arc<PlayerInner>,
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

/// The recorder is configured for a 192 kbps output stream.
pub(crate) fn capture_bytes_for_duration(duration_ms: u64) -> u64 {
    duration_ms.saturating_mul(24_000) / 1000
}

impl PlayerControl {
    /// Records the track a Jellyfin client explicitly made active. Audio URL
    /// probes for other queued tracks must not retune the shared Spotify player.
    pub async fn note_requested(&self, item: uuid::Uuid) {
        *self.inner.requested_item.lock().await = Some(item);
    }

    /// Capture plan for `item`, if it is the one being recorded.
    pub async fn session_for(&self, item: uuid::Uuid) -> Option<CaptureSession> {
        match self.inner.session.lock().await.as_ref() {
            Some(session) if session.item == item => Some(*session),
            _ => None,
        }
    }

    pub(crate) fn stream_started(&self) {
        self.inner.live_streams.fetch_add(1, Ordering::AcqRel);
    }

    pub(crate) fn stream_finished(&self) {
        self.inner.live_streams.fetch_sub(1, Ordering::AcqRel);
    }

    fn has_live_stream(&self) -> bool {
        self.inner.live_streams.load(Ordering::Acquire) != 0
    }

    /// True while some requested track may still be sounding: pausing would
    /// cut a capture short.
    async fn capture_in_progress(&self) -> bool {
        match self.inner.session.lock().await.as_ref() {
            Some(session) => Instant::now() < session.min_end,
            None => false,
        }
    }

    pub fn note_activity(&self) {
        *self.inner.last_activity.lock().unwrap() = Some(Instant::now());
        *self.inner.idle_paused.lock().unwrap() = false;
    }

    /// Long-idle watchdog: pauses the client once no Jellyfin client has asked
    /// for audio for `timeout`, but never mid-capture (pauses land at song
    /// boundaries by construction).
    pub async fn idle_watchdog(
        bridge: BridgeState,
        control: PlayerControl,
        timeout: Duration,
    ) {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let last = *control.inner.last_activity.lock().unwrap();
            let Some(last) = last else { continue };
            if last.elapsed() < timeout || control.capture_in_progress().await {
                continue;
            }
            // Pausing alone loses the race against autoplay advancing between
            // tracks; draining the queue leaves the client with nothing to
            // roll on to.
            let already_paused = *control.inner.idle_paused.lock().unwrap();
            match eval_on_bridge(
                &bridge,
                "Spicetify.Platform.PlayerAPI.clearQueue(); Spicetify.Player.pause(); \"paused\"".to_string(),
            )
            .await
            {
                Ok(_) if !already_paused => eprintln!("playback paused after {}s idle", timeout.as_secs()),
                Ok(_) => {}
                Err(error) => eprintln!("idle pause failed: {error}"),
            }
        }
    }
}

const PLAY_JS: &str = r#"
(async () => {
    const uri = URI_PLACEHOLDER;
    const parts = uri.split(":");
    if (parts[1] === "track") {
        try { Spicetify.Platform.History.push("/track/" + parts[2]); } catch (e) {}
    }
    await Spicetify.Player.playUri(uri);
    return "started";
})()
"#;

const VERIFY_PLAYBACK_JS: &str = r#"
(async () => {
    const state = await Spicetify.Platform.PlayerAPI.getState();
    const current = state?.item?.uri ?? null;
    if (current !== URI_PLACEHOLDER) throw new Error("playing " + current);
    return state?.item?.name ?? "unknown track";
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

async fn verify_playback(bridge: &BridgeState, uri: &str) -> Result<String, String> {
    let literal = serde_json::to_string(uri).map_err(|e| e.to_string())?;
    let code = VERIFY_PLAYBACK_JS.replace("URI_PLACEHOLDER", &literal);
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
pub async fn prepare(
    state: &crate::AppState,
    item_id: uuid::Uuid,
    recording: &std::path::Path,
    cache_dir: &std::path::Path,
    probe: bool,
) -> bool {
    let generation = (!probe).then(|| {
        state
            .player
            .inner
            .switch_generation
            .fetch_add(1, Ordering::AcqRel)
            + 1
    });
    state.player.note_activity();
    let (uri, duration_ms) = {
        let catalog = state.catalog.read().unwrap();
        match catalog.tracks.get(&item_id) {
            Some(track) => (Some(track.uri.clone()), track.duration_ms),
            None => (None, 0),
        }
    };
    let Some(uri) = uri else {
        return false;
    };

    let mut last = state.player.inner.last_item.lock().await;
    if generation.is_some_and(|generation| {
        state.player.inner.switch_generation.load(Ordering::Acquire) != generation
    }) {
        return false;
    }
    let requested = *state.player.inner.requested_item.lock().await;
    if probe && requested.is_some_and(|requested| requested != item_id) {
        return false;
    }
    let still_capturing = state
        .player
        .session_for(item_id)
        .await
        .is_some_and(|session| Instant::now() < session.min_end);
    if *last == Some(item_id) && still_capturing {
        return true;
    }
    // Yuzic preloads the next two queue entries. Their URLs look identical to
    // the active URL, but switching Spotify for one of them would make the
    // current stream contain the wrong track. A real skip closes the old
    // stream first, so use that boundary rather than blocking for the song's
    // full duration.
    if *last != Some(item_id)
        && state.player.has_live_stream()
    {
        return false;
    }
    // Native players probe several queued URLs immediately after a selection.
    // Do not let those probes retune the shared Spotify player; a later request
    // outside this window is treated as an intentional track change.
    if probe
        && *last != Some(item_id)
        && state
            .player
            .inner
            .last_switch
            .lock()
            .await
            .is_some_and(|switched| switched.elapsed() < Duration::from_secs(10))
    {
        return false;
    }

    // Invalidate readers of the old shared recording before resetting it. Their
    // next loop iteration will observe the missing session and stop instead of
    // consuming bytes from the new track.
    *state.player.inner.session.lock().await = None;
    let previous_len = tokio::fs::metadata(recording).await.map(|m| m.len()).unwrap_or(0);
    // Clear the old capture before Spotify starts the new track. Resetting
    // after play has begun loses the intro while waiting for verification.
    reset_recorder();
    wait_recorder_restarted(recording, previous_len).await;
    match play_uri(&state.bridge, &uri).await {
        Ok(title) => {
            if generation.is_some_and(|generation| {
                state.player.inner.switch_generation.load(Ordering::Acquire) != generation
            }) {
                return false;
            }
            let capture_start = Instant::now();
            eprintln!("now playing: {title}");
            tokio::time::sleep(Duration::from_millis(2500)).await;
            let title = match verify_playback(&state.bridge, &uri).await {
                Ok(title) => title,
                Err(error) => {
                    eprintln!("could not verify playback of {uri}: {error}");
                    return true;
                }
            };
            if generation.is_some_and(|generation| {
                state.player.inner.switch_generation.load(Ordering::Acquire) != generation
            }) {
                return false;
            }
            eprintln!("verified now playing: {title}");
            let min_end = capture_start + Duration::from_millis(duration_ms);
            let expected_bytes = capture_bytes_for_duration(duration_ms);
            *state.player.inner.session.lock().await =
                Some(CaptureSession { item: item_id, min_end, expected_bytes });
            *last = Some(item_id);
            *state.player.inner.last_switch.lock().await = Some(Instant::now());
            // Park the finished capture in the cache once the song is over.
            tokio::spawn(save_to_cache(
                recording.to_path_buf(),
                cache_dir.to_path_buf(),
                item_id,
                min_end,
                expected_bytes,
                state.player.clone(),
            ));
        }
        Err(error) => eprintln!("could not play {uri}: {error}"),
    }
    true
}

/// A few seconds of tail may be chopped off by the recorder's flush buffer;
/// more than that and the capture is not worth keeping.
const CACHE_TAIL_TOLERANCE_BYTES: u64 = 6 * 24_000;

async fn save_to_cache(
    recording: std::path::PathBuf,
    cache_dir: std::path::PathBuf,
    item_id: uuid::Uuid,
    ready_at: Instant,
    expected_bytes: u64,
    control: PlayerControl,
) {
    while Instant::now() < ready_at {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    // ffmpeg flushes in bursts; the tail of this song often only leaves its
    // buffer once following audio pushes it out (or not at all if nothing
    // follows). Wait for completeness, then give up gracefully.
    let deadline = ready_at + Duration::from_secs(45);
    loop {
        let len = tokio::fs::metadata(&recording).await.map(|m| m.len()).unwrap_or(0);
        if len >= expected_bytes || Instant::now() >= deadline {
            break;
        }
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
    // Serialize the snapshot with recorder resets. Otherwise a later track
    // can replace the shared file while this task still believes it is saving
    // the previous capture.
    let last_item = control.inner.last_item.lock().await;
    if *last_item != Some(item_id) {
        return;
    }
    let Ok(raw) = tokio::fs::read(&recording).await else {
        return;
    };
    let take = (raw.len() as u64).min(expected_bytes) as usize;
    if expected_bytes.saturating_sub(take as u64) > CACHE_TAIL_TOLERANCE_BYTES {
        eprintln!("capture of {item_id} incomplete ({} of {expected_bytes} bytes), not cached", raw.len());
        return;
    }
    let tmp = cache_dir.join(format!("tmp-{item_id}.aac"));
    let final_path = cache_dir.join(format!("{item_id}.aac"));
    if tokio::fs::write(&tmp, &raw[..take]).await.is_ok() {
        let _ = tokio::fs::rename(&tmp, &final_path).await;
    }
}
