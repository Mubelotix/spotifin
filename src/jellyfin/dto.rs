use serde::Serialize;
use uuid::Uuid;

use crate::catalog::{Catalog, Item};

const TICKS_PER_MS: u64 = 10_000;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_record_count: usize,
    pub start_index: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IdName {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageTags {
    pub primary: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserItemData {
    pub item_id: Uuid,
    pub is_favorite: bool,
    pub played: bool,
    pub play_count: u32,
    pub playback_position_ticks: u64,
    pub last_played_date: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDto {
    pub id: Uuid,
    pub name: Option<String>,
    #[serde(rename = "Type")]
    pub kind: &'static str,
    pub is_folder: Option<bool>,
    pub media_type: Option<&'static str>,
    pub sort_name: Option<String>,
    pub child_count: Option<usize>,
    pub index_number: Option<u32>,
    pub parent_index_number: Option<u32>,
    pub album: Option<String>,
    pub album_id: Option<Uuid>,
    pub artists: Option<Vec<String>>,
    pub artist_items: Option<Vec<IdName>>,
    pub album_artist: Option<String>,
    pub album_artists: Option<Vec<IdName>>,
    pub run_time_ticks: Option<u64>,
    pub container: Option<&'static str>,
    pub production_year: Option<i32>,
    pub image_tags: Option<ImageTags>,
    pub user_data: Option<UserItemData>,
    pub playlist_item_id: Option<Uuid>,
}

pub fn user_data(catalog: &Catalog, id: Uuid) -> UserItemData {
    let count = catalog.play_count(id);
    UserItemData {
        item_id: id,
        is_favorite: catalog.is_favorite(id),
        played: count > 0,
        play_count: count,
        playback_position_ticks: catalog.position_ticks(id),
        last_played_date: catalog
            .last_played(id)
            .map(|ms| chrono_like_iso(ms)),
    }
}

/// RFC 3339 with second precision; no chrono dependency for one format.
fn chrono_like_iso(ms: i64) -> String {
    const SECONDS_PER_DAY: i64 = 86_400;
    let secs = ms / 1000;
    let days = secs / SECONDS_PER_DAY;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        secs % SECONDS_PER_DAY / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// Days-since-epoch to civil date (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn artist_refs(catalog: &Catalog, ids: &[Uuid]) -> (Vec<String>, Vec<IdName>) {
    let names = ids.iter().filter_map(|id| catalog.artists.get(id).map(|a| a.name.clone())).collect();
    let refs = ids
        .iter()
        .filter_map(|id| catalog.artists.get(id).map(|a| IdName { id: a.id, name: a.name.clone() }))
        .collect();
    (names, refs)
}

/// The image tag doubles as the Spotify image reference so `/Images/Primary`
/// can resolve it without extra lookups.
fn image_tag(image: &Option<String>) -> Option<ImageTags> {
    let encoded = image.as_ref().map(|url| encode_image_tag(url))?;
    Some(ImageTags { primary: encoded })
}

fn encode_image_tag(url: &str) -> String {
    // spotify:image:<hex> or a full https URL; keep it URL-safe.
    url.replace("spotify:image:", "img_").replace([':', '/', '?', '='], "_")
}

pub fn base_item(catalog: &Catalog, item: &Item<'_>, playlist_item_id: Option<Uuid>) -> BaseItemDto {
    let mut dto = BaseItemDto {
        id: item.id(),
        name: Some(item.name().to_string()),
        kind: item.jellyfin_type(),
        sort_name: Some(item.name().to_string()),
        is_folder: Some(item.is_folder()),
        media_type: matches!(item, Item::Track(_)).then_some("Audio"),
        child_count: None,
        index_number: None,
        parent_index_number: None,
        album: None,
        album_id: None,
        artists: None,
        artist_items: None,
        album_artist: None,
        album_artists: None,
        run_time_ticks: None,
        container: None,
        production_year: None,
        image_tags: None,
        user_data: Some(user_data(catalog, item.id())),
        playlist_item_id,
    };
    match item {
        Item::Track(track) => fill_track(catalog, &mut dto, track),
        Item::Album(album) => fill_album(catalog, &mut dto, album),
        Item::Playlist(playlist) => {
            dto.child_count = Some(playlist.entries.len());
            dto.image_tags = image_tag(&playlist.image);
        }
        _ => {}
    }
    dto
}

fn fill_track(catalog: &Catalog, dto: &mut BaseItemDto, track: &crate::catalog::Track) {
    let (artists, artist_items) = artist_refs(catalog, &track.artist_ids);
    dto.artists = Some(artists);
    dto.artist_items = Some(artist_items);
    dto.index_number = Some(track.index);
    dto.parent_index_number = Some(track.disc);
    dto.run_time_ticks = Some(track.duration_ms * TICKS_PER_MS);
    dto.container = Some("mp3");
    if let Some(album) = track.album_id.and_then(|id| catalog.albums.get(&id)) {
        dto.album = Some(album.name.clone());
        dto.album_id = Some(album.id);
        let (album_artists, _) = artist_refs(catalog, &album.artist_ids);
        dto.album_artist = album_artists.first().cloned();
        dto.album_artists = Some(
            album.artist_ids.iter().filter_map(|id| catalog.artists.get(id).map(|a| IdName { id: a.id, name: a.name.clone() })).collect(),
        );
        dto.image_tags = image_tag(&album.image);
    }
}

fn fill_album(catalog: &Catalog, dto: &mut BaseItemDto, album: &crate::catalog::Album) {
    let count = catalog.tracks.values().filter(|t| t.album_id == Some(album.id)).count();
    dto.child_count = Some(count);
    dto.production_year = album.year;
    dto.image_tags = image_tag(&album.image);
    let (names, refs) = artist_refs(catalog, &album.artist_ids);
    dto.album_artist = names.first().cloned();
    dto.album_artists = Some(refs);
}
