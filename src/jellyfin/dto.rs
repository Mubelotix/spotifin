use serde::Serialize;
use std::path::{Path, PathBuf};
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
    pub key: String,
    pub is_favorite: bool,
    pub likes: Option<bool>,
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
    pub collection_type: Option<&'static str>,
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
    pub has_lyrics: Option<bool>,
    pub production_year: Option<i32>,
    pub image_tags: Option<ImageTags>,
    pub user_data: Option<UserItemData>,
    pub playlist_item_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub media_sources: Option<Vec<MediaSourceDto>>,
    pub media_streams: Option<Vec<MediaStreamDto>>,
    pub path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaSourceDto {
    pub id: Uuid,
    pub protocol: &'static str,
    #[serde(rename = "Type")]
    pub source_type: &'static str,
    pub container: &'static str,
    pub is_remote: bool,
    pub supports_transcoding: bool,
    pub supports_direct_stream: bool,
    pub supports_direct_play: bool,
    pub is_infinite_stream: bool,
    pub requires_opening: bool,
    pub requires_closing: bool,
    pub requires_looping: bool,
    pub supports_probing: bool,
    pub read_at_native_framerate: bool,
    pub ignore_dts: bool,
    pub ignore_index: bool,
    pub gen_pts_input: bool,
    pub media_streams: Vec<MediaStreamDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaStreamDto {
    pub index: u8,
    #[serde(rename = "Type")]
    pub stream_type: &'static str,
    pub codec: &'static str,
    pub bit_rate: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub is_interlaced: bool,
    pub is_default: bool,
    pub is_forced: bool,
    pub is_external: bool,
    pub is_text_subtitle_stream: bool,
    pub supports_external_stream: bool,
}

pub fn lyrics_cache_path(id: Uuid) -> PathBuf {
    std::env::var_os("AUDIO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("data/audio").to_path_buf())
        .join("cache")
        .join(format!("lyrics-{id}.json"))
}

pub fn user_data(catalog: &Catalog, id: Uuid) -> UserItemData {
    let count = catalog.play_count(id);
    UserItemData {
        item_id: id,
        key: id.to_string(),
        is_favorite: catalog.is_favorite(id),
        likes: catalog.likes(id),
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
        collection_type: match item {
            Item::Library(_) => Some("music"),
            Item::PlaylistLibrary => Some("playlists"),
            _ => None,
        },
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
        has_lyrics: None,
        production_year: None,
        image_tags: None,
        user_data: Some(user_data(catalog, item.id())),
        playlist_item_id,
        parent_id: None,
        media_sources: None,
        media_streams: None,
        path: None,
    };
    match item {
        Item::Track(track) => fill_track(catalog, &mut dto, track),
        Item::Album(album) => fill_album(catalog, &mut dto, album),
        Item::Playlist(playlist) => {
            dto.child_count = Some(playlist.entries.len());
            dto.run_time_ticks = Some(
                catalog
                    .playlist_tracks(playlist)
                    .iter()
                    .filter_map(|item| match item {
                        Item::Track(track) => Some(track.duration_ms),
                        _ => None,
                    })
                    .sum::<u64>()
                    * TICKS_PER_MS,
            );
            dto.image_tags = image_tag(&playlist.image);
            dto.path = Some(format!("/data/playlists/{}", playlist.id));
        }
        Item::Artist(artist) => {
            // Fintunes renders artist collections without guarding these
            // fields, even though Jellyfin may omit them for artists.
            dto.artists = Some(Vec::new());
            dto.artist_items = Some(Vec::new());
            dto.album_artists = Some(Vec::new());
            // Keep the field non-null as well; Fintunes treats collection
            // metadata as required while navigating from the player.
            dto.image_tags = Some(ImageTags {
                primary: artist.image.as_deref().map(encode_image_tag).unwrap_or_default(),
            });
            dto.run_time_ticks = Some(0);
        }
        _ => {}
    }
    dto
}

fn fill_track(catalog: &Catalog, dto: &mut BaseItemDto, track: &crate::catalog::Track) {
    let (artists, artist_items) = artist_refs(catalog, &track.artist_ids);
    dto.artists = Some(artists);
    dto.artist_items = Some(artist_items);
    // Fintunes' player model treats these collection fields as required,
    // including for tracks whose album has not been resolved yet.
    dto.album_artists = Some(Vec::new());
    dto.image_tags = Some(ImageTags { primary: String::new() });
    dto.index_number = Some(track.index);
    dto.parent_index_number = Some(track.disc);
    dto.run_time_ticks = Some(track.duration_ms * TICKS_PER_MS);
    dto.container = Some("aac");
    dto.media_streams = Some(vec![MediaStreamDto {
        index: 0,
        stream_type: "Audio",
        codec: "aac",
        bit_rate: 192_000,
        sample_rate: 44_100,
        channels: 2,
        is_interlaced: false,
        is_default: true,
        is_forced: false,
        is_external: false,
        is_text_subtitle_stream: false,
        supports_external_stream: false,
    }]);
    dto.media_sources = Some(vec![MediaSourceDto {
        id: track.id,
        protocol: "Http",
        source_type: "Default",
        container: "aac",
        is_remote: false,
        supports_transcoding: true,
        supports_direct_stream: true,
        supports_direct_play: true,
        is_infinite_stream: false,
        requires_opening: false,
        requires_closing: false,
        requires_looping: false,
        supports_probing: false,
        read_at_native_framerate: false,
        ignore_dts: false,
        ignore_index: false,
        gen_pts_input: false,
        media_streams: vec![MediaStreamDto {
            index: 0,
            stream_type: "Audio",
            codec: "aac",
            bit_rate: 192_000,
            sample_rate: 44_100,
            channels: 2,
            is_interlaced: false,
            is_default: true,
            is_forced: false,
            is_external: false,
            is_text_subtitle_stream: false,
            supports_external_stream: false,
        }],
    }]);
    dto.has_lyrics = Some(match std::fs::read(lyrics_cache_path(track.id)) {
        Ok(raw) => serde_json::from_slice::<serde_json::Value>(&raw)
            .ok()
            .and_then(|json| json.get("Lyrics")?.as_array().map(|lyrics| !lyrics.is_empty()))
            .unwrap_or(true),
        Err(_) => true,
    });
    if let Some(album) = track.album_id.and_then(|id| catalog.albums.get(&id)) {
        dto.album = Some(album.name.clone());
        dto.album_id = Some(album.id);
        dto.production_year = album.year;
        let (album_artists, _) = artist_refs(catalog, &album.artist_ids);
        dto.album_artist = album_artists.first().cloned();
        dto.album_artists = Some(
            album.artist_ids.iter().filter_map(|id| catalog.artists.get(id).map(|a| IdName { id: a.id, name: a.name.clone() })).collect(),
        );
        dto.image_tags = image_tag(&album.image);
    }
}

fn fill_album(catalog: &Catalog, dto: &mut BaseItemDto, album: &crate::catalog::Album) {
    let tracks: Vec<_> = catalog.tracks.values().filter(|t| t.album_id == Some(album.id)).collect();
    let (artists, artist_items) = artist_refs(catalog, &album.artist_ids);
    dto.artists = Some(artists);
    dto.artist_items = Some(artist_items);
    dto.run_time_ticks = Some(tracks.iter().map(|track| track.duration_ms).sum::<u64>() * TICKS_PER_MS);
    let count = tracks.len();
    dto.child_count = Some(count);
    dto.production_year = album.year;
    dto.image_tags = image_tag(&album.image);
    let (names, refs) = artist_refs(catalog, &album.artist_ids);
    dto.album_artist = names.first().cloned();
    dto.album_artists = Some(refs);
}
