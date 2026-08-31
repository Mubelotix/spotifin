pub mod auth;
pub mod dto;
pub mod images;
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
        auth::public_users,
        auth::get_user,
        auth::get_me,
        auth::logout,
        items::views,
        items::user_views,
        items::user_items,
        items::items,
        items::user_item,
        items::item_detail,
        items::instant_mix,
        playlists::get_playlist,
        playlists::playlist_items,
        playlists::create_playlist,
        playlists::add_to_playlist,
        playlists::remove_from_playlist,
        images::image,
        images::image_index,
        user_data::get_user_data,
        user_data::post_user_data,
        user_data::mark_favorite,
        user_data::unmark_favorite,
        user_data::mark_played,
        user_data::unmark_played,
        playback::playback_info_get,
        playback::playback_info_post,
        playback::playing_started,
        playback::playing_progress,
        playback::playing_stopped,
        playback::playing_ping,
    ]
}
