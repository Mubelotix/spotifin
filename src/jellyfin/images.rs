use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::{get, State};
use std::io::Cursor;
use uuid::Uuid;

use crate::AppState;

pub struct ImageResponse(Response<'static>);

impl<'r> Responder<'r, 'static> for ImageResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(self.0)
    }
}

fn image_url_of(state: &State<AppState>, item_id: Uuid) -> Option<String> {
    let catalog = state.catalog.read().unwrap();
    let item = catalog.item(item_id)?;
    match item {
        crate::catalog::Item::Playlist(playlist) => playlist.image.clone(),
        crate::catalog::Item::Album(album) => album.image.clone(),
        crate::catalog::Item::Track(track) => {
            let album = track.album_id.and_then(|id| catalog.albums.get(&id))?;
            album.image.clone()
        }
        _ => None,
    }
}

/// Spotify image references look like `spotify:image:<hex>`; the CDN URL is
/// deterministic, so artwork is served as a redirect.
fn cdn_url(image: &str) -> Option<String> {
    if let Some(hex) = image.strip_prefix("spotify:image:") {
        if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        return Some(format!("https://i.scdn.co/image/{hex}"));
    }
    // Search results already come with full CDN URLs.
    image.starts_with("https://").then(|| image.to_string())
}

#[get("/Items/<item_id>/Images/<image_type>")]
pub async fn image(item_id: Uuid, image_type: &str, state: &State<AppState>) -> Result<ImageResponse, Status> {
    primary_image(item_id, image_type, state).await
}

#[get("/Items/<item_id>/Images/<image_type>/<index>")]
pub async fn image_index(item_id: Uuid, image_type: &str, index: usize, state: &State<AppState>) -> Result<ImageResponse, Status> {
    let _ = index;
    primary_image(item_id, image_type, state).await
}

async fn primary_image(item_id: Uuid, image_type: &str, state: &State<AppState>) -> Result<ImageResponse, Status> {
    if !image_type.eq_ignore_ascii_case("primary") && !image_type.eq_ignore_ascii_case("backdrop") {
        return Err(Status::NotFound);
    }
    let image = image_url_of(state, item_id).ok_or(Status::NotFound)?;
    let url = cdn_url(&image).ok_or(Status::NotFound)?;
    let bytes = tokio::process::Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location", &url])
        .output()
        .await
        .map_err(|_| Status::BadGateway)?;
    if !bytes.status.success() {
        return Err(Status::BadGateway);
    }
    let mut response = Response::build();
    response.header(rocket::http::ContentType::JPEG);
    response.sized_body(bytes.stdout.len(), Cursor::new(bytes.stdout));
    Ok(ImageResponse(response.finalize()))
}
