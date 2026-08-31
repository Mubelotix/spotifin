use rocket::http::{uri::Absolute, Status};
use rocket::response::Redirect;
use rocket::{get, State};
use uuid::Uuid;

use crate::AppState;

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
    let hex = image.strip_prefix("spotify:image:")?;
    if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("https://i.scdn.co/image/{hex}"))
}

#[get("/Items/<item_id>/Images/<image_type>")]
pub fn image(item_id: Uuid, image_type: &str, state: &State<AppState>) -> Result<Redirect, Status> {
    primary_image(item_id, image_type, state)
}

#[get("/Items/<item_id>/Images/<image_type>/<index>")]
pub fn image_index(item_id: Uuid, image_type: &str, index: usize, state: &State<AppState>) -> Result<Redirect, Status> {
    let _ = index;
    primary_image(item_id, image_type, state)
}

fn primary_image(item_id: Uuid, image_type: &str, state: &State<AppState>) -> Result<Redirect, Status> {
    if !image_type.eq_ignore_ascii_case("primary") && !image_type.eq_ignore_ascii_case("backdrop") {
        return Err(Status::NotFound);
    }
    let image = image_url_of(state, item_id).ok_or(Status::NotFound)?;
    let url = cdn_url(&image).ok_or(Status::NotFound)?;
    let absolute = Absolute::parse_owned(url).map_err(|_| Status::NotFound)?;
    Ok(Redirect::to(absolute))
}
