pub mod auth;
pub mod control;
pub mod dto;
pub mod images;
pub mod lyrics;
pub mod items;
pub mod playback;
pub mod playlists;
pub mod user_data;

use rocket::Route;

/// All Jellyfin-compatible routes; mounted at `/` since clients append
/// `/api` to the server URL and Rocket strips nothing.
pub fn routes() -> Vec<Route> {
    rocket::routes![
        auth::authenticate,
        auth::web_client_bootstrap,
        auth::public_system_info,
        auth::system_info,
        auth::ping,
        auth::public_users,
        auth::get_user,
        auth::get_me,
        auth::logout,
        control::capabilities,
        control::sessions,
        control::playstate,
        control::socket,
        items::views,
        items::user_views,
        items::media_folders,
        items::user_items,
        items::items,
        items::user_item,
        items::item_detail,
        items::album_artists,
        items::artist_by_name,
        items::all_artists,
        items::genres,
        items::music_genres,
        items::search_hints,
        items::instant_mix,
        items::similar,
        playlists::playlists,
        playlists::get_playlist,
        playlists::playlist_items,
        playlists::create_playlist,
        playlists::add_to_playlist,
        playlists::remove_from_playlist,
        playlists::move_entry,
        images::image,
        images::image_index,
        user_data::get_user_data,
        user_data::post_user_data,
        user_data::get_user_data_modern,
        user_data::post_user_data_modern,
        user_data::set_rating,
        user_data::clear_rating,
        user_data::set_rating_legacy,
        user_data::clear_rating_legacy,
        user_data::mark_favorite,
        user_data::unmark_favorite,
        user_data::mark_favorite_modern,
        user_data::unmark_favorite_modern,
        user_data::mark_played,
        user_data::unmark_played,
        lyrics::lyrics,
        playback::playback_info_get,
        playback::playback_info_post,
        playback::playing_started,
        playback::playing_progress,
        playback::playing_stopped,
        playback::playing_ping,
    ]
}
