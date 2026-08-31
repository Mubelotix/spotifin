mod audio;
mod bridge;
mod catalog;
mod jellyfin;
mod player;
mod spotify;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rocket::get;
use rocket::routes;

pub struct AudioPaths {
    recording: Arc<PathBuf>,
    hls: Arc<PathBuf>,
    cache: Arc<PathBuf>,
}

pub struct AppState {
    audio: AudioPaths,
    pub catalog: std::sync::Arc<std::sync::RwLock<catalog::Catalog>>,
    pub bridge: bridge::BridgeState,
    pub player: player::PlayerControl,
}

fn data_dir() -> PathBuf {
    std::env::var_os("AUDIO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/audio"))
}

#[get("/health")]
fn health() -> rocket::http::Status {
    rocket::http::Status::Ok
}

/// Keeps the catalog fresh: retries until the bridge is up, then refreshes
/// every REFRESH_INTERVAL. A failed refresh just waits for the next tick.
const REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60);

/// Right after a container boot the client answers with half-empty views;
/// a collection much smaller than the previous one is rejected and retried
/// quickly instead of replacing a good catalog.
const SUSPECT_SHRINK_FACTOR: usize = 4;
const SUSPECT_MAX_TRIES: u32 = 5;

async fn refresh_loop(bridge: bridge::BridgeState, catalog: std::sync::Arc<std::sync::RwLock<catalog::Catalog>>) {
    let mut known_tracks = 0usize;
    let mut suspect_streak = 0u32;
    loop {
        match spotify::collect(&bridge).await {
            Ok(fresh) => {
                let size = fresh.tracks.len();
                let suspect =
                    known_tracks > 0 && size * SUSPECT_SHRINK_FACTOR < known_tracks
                        && suspect_streak < SUSPECT_MAX_TRIES;
                if suspect {
                    suspect_streak += 1;
                    eprintln!(
                        "catalog looks partial ({size} tracks, had {known_tracks}), retrying"
                    );
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    continue;
                }
                suspect_streak = 0;
                known_tracks = size;
                catalog.write().unwrap().merge(fresh);
                eprintln!("catalog refreshed: {size} tracks");
                tokio::time::sleep(REFRESH_INTERVAL).await;
            }
            Err(error) => {
                eprintln!("catalog refresh pending: {error}");
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

fn audio_and_jellyfin() -> Vec<rocket::Route> {
    let mut all = audio::routes();
    all.extend(jellyfin::routes());
    all
}

#[rocket::launch]
async fn rocket() -> _ {
    let root = data_dir();
    let hls = root.join("hls");
    let cache = root.join("cache");
    let recording = root.join("recording.aac");
    let state = AppState {
        audio: AudioPaths {
            recording: Arc::new(recording),
            hls: Arc::new(hls.clone()),
            cache: Arc::new(cache.clone()),
        },
        catalog: Arc::default(),
        bridge: bridge::BridgeState::default(),
        player: player::PlayerControl::default(),
    };

    if let Err(error) = tokio::fs::create_dir_all(&hls).await {
        eprintln!("could not create hls directory: {error}");
    }
    if let Err(error) = tokio::fs::create_dir_all(&cache).await {
        eprintln!("could not create cache directory: {error}");
    }

    tokio::spawn(refresh_loop(state.bridge.clone(), state.catalog.clone()));
    let idle_timeout = Duration::from_secs(
        std::env::var("PLAYBACK_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(600),
    );
    tokio::spawn(player::PlayerControl::idle_watchdog(state.bridge.clone(), state.player.clone(), idle_timeout));

    // Clients append /api to the server URL; serve under both prefixes.
    rocket::build()
        .manage(state)
        .mount("/", routes![health, bridge::ws, bridge::debug_eval])
        .mount("/api", audio_and_jellyfin())
        .mount("/", audio_and_jellyfin())
}
