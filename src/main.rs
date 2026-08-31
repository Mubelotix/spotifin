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

async fn refresh_loop(bridge: bridge::BridgeState, catalog: std::sync::Arc<std::sync::RwLock<catalog::Catalog>>) {
    loop {
        match spotify::collect(&bridge).await {
            Ok(fresh) => {
                catalog.write().unwrap().merge(fresh);
                eprintln!("catalog refreshed");
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
    let recording = root.join("recording.aac");
    let state = AppState {
        audio: AudioPaths {
            recording: Arc::new(recording.clone()),
            hls: Arc::new(hls.clone()),
        },
        catalog: Arc::default(),
        bridge: bridge::BridgeState::default(),
        player: player::PlayerControl::default(),
    };

    if let Err(error) = tokio::fs::create_dir_all(&hls).await {
        eprintln!("could not create hls directory: {error}");
    }

    tokio::spawn(refresh_loop(state.bridge.clone(), state.catalog.clone()));

    // Clients append /api to the server URL; serve under both prefixes.
    rocket::build()
        .manage(state)
        .mount("/", routes![health, bridge::ws, bridge::debug_eval])
        .mount("/api", audio_and_jellyfin())
        .mount("/", audio_and_jellyfin())
}
