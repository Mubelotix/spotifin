use std::collections::HashMap;
use std::collections::HashSet;

use uuid::Uuid;

pub const NAMESPACE: Uuid = Uuid::from_bytes([
    0x8c, 0x5b, 0x1f, 0x42, 0x7a, 0x9e, 0x4d, 0x63, 0xb2, 0x11, 0x3f, 0x08, 0xd6, 0xa4, 0xc7, 0x5e,
]);

pub fn stable_id(key: &str) -> Uuid {
    Uuid::new_v5(&NAMESPACE, key.as_bytes())
}

#[derive(Clone)]
pub struct Artist {
    pub id: Uuid,
    pub uri: String,
    pub name: String,
    pub image: Option<String>,
    pub discography_loaded: bool,
}

#[derive(Clone)]
pub struct Album {
    pub id: Uuid,
    pub uri: String,
    pub name: String,
    pub artist_ids: Vec<Uuid>,
    pub image: Option<String>,
    pub year: Option<i32>,
}

#[derive(Clone)]
pub struct Track {
    pub id: Uuid,
    /// Source Spotify URI, used to drive playback through the client.
    pub uri: String,
    pub name: String,
    pub album_id: Option<Uuid>,
    pub artist_ids: Vec<Uuid>,
    pub index: u32,
    pub disc: u32,
    pub duration_ms: u64,
}

#[derive(Clone)]
pub struct PlaylistEntry {
    pub id: Uuid,
    /// Spotify row uid when the playlist is client-backed; drives reorder ops.
    pub uid: Option<String>,
    pub track_id: Uuid,
}

#[derive(Clone)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    /// Jellyfin visibility flag; playlists are private by default.
    pub is_public: bool,
    pub image: Option<String>,
    /// Spotify URI for client-backed playlists; None for ephemeral ones.
    pub spotify_uri: Option<String>,
    /// Backing Spotify playlists for synthesized collections such as Daily Mixes.
    pub source_uris: Vec<String>,
    pub loaded: bool,
    pub entries: Vec<PlaylistEntry>,
}

/// Unified read-only view over every kind of catalog item, used by the
/// query engine and DTO builder.
#[derive(Clone, Copy)]
pub enum Item<'a> {
    Library(&'a str),
    PlaylistLibrary,
    Track(&'a Track),
    Album(&'a Album),
    Artist(&'a Artist),
    Playlist(&'a Playlist),
}

impl Item<'_> {
    pub fn id(&self) -> Uuid {
        match self {
            Item::Library(_) => library_id(),
            Item::PlaylistLibrary => playlist_library_id(),
            Item::Track(t) => t.id,
            Item::Album(a) => a.id,
            Item::Artist(a) => a.id,
            Item::Playlist(p) => p.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Item::Library(name) => name,
            Item::PlaylistLibrary => PLAYLIST_LIBRARY_NAME,
            Item::Track(t) => &t.name,
            Item::Album(a) => &a.name,
            Item::Artist(a) => &a.name,
            Item::Playlist(p) => &p.name,
        }
    }

    pub fn jellyfin_type(&self) -> &'static str {
        match self {
            Item::Library(_) => "CollectionFolder",
            Item::PlaylistLibrary => "ManualPlaylistsFolder",
            Item::Track(_) => "Audio",
            Item::Album(_) => "MusicAlbum",
            Item::Artist(_) => "MusicArtist",
            Item::Playlist(_) => "Playlist",
        }
    }

    pub fn is_folder(&self) -> bool {
        !matches!(self, Item::Track(_))
    }
}

pub fn library_id() -> Uuid {
    stable_id("library:music")
}

pub fn playlist_library_id() -> Uuid {
    stable_id("library:playlists")
}

pub const PLAYLIST_LIBRARY_NAME: &str = "Playlists";

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Jellyfin-style search: every whitespace-separated word must match somewhere
/// in the track's name, album or artists ("darude sandstorm" finds "Sandstorm"
/// by Darude).
trait ContainsAllWords {
    fn contains_all_words(&self, term: &str) -> bool;
}

impl ContainsAllWords for String {
    fn contains_all_words(&self, term: &str) -> bool {
        let haystack = self.as_str();
        term.split_whitespace().all(|word| haystack.contains(&word.to_lowercase()))
    }
}

pub const LIBRARY_NAME: &str = "Music";

#[derive(Default)]
pub struct Catalog {
    pub artists: HashMap<Uuid, Artist>,
    pub albums: HashMap<Uuid, Album>,
    pub tracks: HashMap<Uuid, Track>,
    pub playlists: HashMap<Uuid, Playlist>,
    /// Albums the user explicitly saved; the only ones listed as browsable.
    pub saved_albums: HashSet<Uuid>,
    /// Artists the user follows; the only ones listed as browsable.
    pub followed_artists: HashSet<Uuid>,
    /// Tracks learned from remote search; retained for direct playback but not
    /// shown as part of the browsable library.
    pub remote_tracks: HashSet<Uuid>,
    favorites: HashSet<Uuid>,
    likes: HashMap<Uuid, bool>,
    play_counts: HashMap<Uuid, u32>,
    /// Last reported playback position in ticks; the Spotify client owns the
    /// real cursor, we only remember what clients tell us.
    positions: HashMap<Uuid, u64>,
    last_played: HashMap<Uuid, i64>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adopts the freshly collected catalog while preserving runtime state:
    /// favorites, play counts, and dynamically ingested items (e.g. search
    /// results) that the collector does not know about.
    pub fn merge(&mut self, mut fresh: Catalog) {
        let old_favorites = std::mem::take(&mut self.favorites);
        fresh.favorites.extend(old_favorites.into_iter().filter(|id| !fresh.tracks.contains_key(id)));
        let old_likes = std::mem::take(&mut self.likes);
        fresh.likes.extend(old_likes.into_iter().filter(|(id, _)| !fresh.tracks.contains_key(id)));
        let old_remote_tracks = std::mem::take(&mut self.remote_tracks);
        fresh.remote_tracks.extend(old_remote_tracks.into_iter().filter(|id| !fresh.tracks.contains_key(id)));
        fresh.play_counts = std::mem::take(&mut self.play_counts);
        fresh.positions = std::mem::take(&mut self.positions);
        fresh.last_played = std::mem::take(&mut self.last_played);
        for (id, track) in std::mem::take(&mut self.tracks) {
            if track.uri.starts_with("spotify:local:") {
                continue;
            }
            fresh.tracks.entry(id).or_insert(track);
        }
        for (id, album) in std::mem::take(&mut self.albums) {
            fresh.albums.entry(id).or_insert(album);
        }
        for (id, artist) in std::mem::take(&mut self.artists) {
            if let Some(current) = fresh.artists.get_mut(&id) {
                if current.image.is_none() {
                    current.image = artist.image;
                }
                current.discography_loaded |= artist.discography_loaded;
            } else {
                fresh.artists.insert(id, artist);
            }
        }
        *self = fresh;
    }

    pub fn item(&self, id: Uuid) -> Option<Item<'_>> {
        if id == library_id() {
            return Some(Item::Library(LIBRARY_NAME));
        }
        if id == playlist_library_id() {
            return Some(Item::PlaylistLibrary);
        }
        if let Some(t) = self.tracks.get(&id) {
            return Some(Item::Track(t));
        }
        if let Some(a) = self.albums.get(&id) {
            return Some(Item::Album(a));
        }
        if let Some(a) = self.artists.get(&id) {
            return Some(Item::Artist(a));
        }
        self.playlists.get(&id).map(Item::Playlist)
    }

    pub fn is_favorite(&self, id: Uuid) -> bool {
        self.favorites.contains(&id) || self.saved_albums.contains(&id) || self.followed_artists.contains(&id)
    }

    pub fn set_favorite(&mut self, id: Uuid, favorite: bool) {
        if self.albums.contains_key(&id) {
            if favorite {
                self.saved_albums.insert(id);
            } else {
                self.saved_albums.remove(&id);
            }
        } else if self.artists.contains_key(&id) {
            if favorite {
                self.followed_artists.insert(id);
            } else {
                self.followed_artists.remove(&id);
            }
        } else if favorite {
            self.favorites.insert(id);
        } else {
            self.favorites.remove(&id);
        }
    }

    pub fn likes(&self, id: Uuid) -> Option<bool> {
        self.likes.get(&id).copied()
    }

    pub fn set_likes(&mut self, id: Uuid, likes: Option<bool>) {
        if let Some(likes) = likes {
            self.likes.insert(id, likes);
        } else {
            self.likes.remove(&id);
        }
    }

    pub fn play_count(&self, id: Uuid) -> u32 {
        self.play_counts.get(&id).copied().unwrap_or(0)
    }

    pub fn position_ticks(&self, id: Uuid) -> u64 {
        self.positions.get(&id).copied().unwrap_or(0)
    }

    pub fn last_played(&self, id: Uuid) -> Option<i64> {
        self.last_played.get(&id).copied()
    }

    pub fn note_progress(&mut self, id: Uuid, ticks: u64) {
        self.positions.insert(id, ticks);
        self.last_played.entry(id).or_insert_with(now_millis);
    }

    pub fn note_stopped(&mut self, id: Uuid, ticks: u64) {
        let finished = ticks > 0;
        self.note_progress(id, ticks);
        if finished {
            *self.play_counts.entry(id).or_insert(0) += 1;
            self.last_played.insert(id, now_millis());
        }
    }

    pub fn note_started(&mut self, id: Uuid) {
        self.note_progress(id, 0);
    }

    pub fn bump_play_count(&mut self, id: Uuid) {
        *self.play_counts.entry(id).or_insert(0) += 1;
    }

    pub fn mark_remote_track(&mut self, id: Uuid) {
        self.remote_tracks.insert(id);
    }

    /// Direct children of `parent`; with the whole catalog when recursive.
    pub fn query(
        &self,
        types: &[&str],
        parent: Option<Uuid>,
        search: Option<&str>,
        favorites_only: bool,
        artist_ids: &[Uuid],
        album_artist_ids: &[Uuid],
        contributing_artist_ids: &[Uuid],
        album_ids: &[Uuid],
    ) -> Vec<Item<'_>> {
        let mut items = self.candidate_items(parent);
        items.retain(|item| {
            self.matches(item, types, parent, search, favorites_only, artist_ids, album_artist_ids, contributing_artist_ids, album_ids)
        });
        if !matches!(parent, Some(id) if self.playlists.contains_key(&id)) {
            items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
        }
        items
    }

    fn candidate_items(&self, parent: Option<Uuid>) -> Vec<Item<'_>> {
        match parent {
            Some(id) if id == playlist_library_id() => self.playlists.values().map(Item::Playlist).collect(),
            Some(id) if id != library_id() => match self.item(id) {
                Some(Item::Album(_)) => self.track_children(|t| t.album_id == Some(id)),
                Some(Item::Artist(_)) => {
                    let mut items: Vec<Item<'_>> = self
                        .albums
                        .values()
                        .filter(|a| a.artist_ids.contains(&id))
                        .map(Item::Album)
                        .collect();
                    items.extend(
                        self.tracks
                            .values()
                            .filter(|t| t.artist_ids.contains(&id))
                            .map(Item::Track),
                    );
                    items
                }
                Some(Item::Playlist(playlist)) => self.playlist_tracks(playlist),
                _ => vec![],
            },
            _ => self.all_items(),
        }
    }

    fn all_items(&self) -> Vec<Item<'_>> {
        let mut items = vec![Item::Library(LIBRARY_NAME), Item::PlaylistLibrary];
        items.extend(self.artists.values().map(Item::Artist));
        items.extend(self.albums.values().map(Item::Album));
        items.extend(self.playlists.values().map(Item::Playlist));
        items.extend(self.tracks.values().map(Item::Track));
        items
    }

    fn track_children(&self, keep: impl Fn(&Track) -> bool) -> Vec<Item<'_>> {
        let mut tracks: Vec<Item<'_>> = self.tracks.values().filter(|t| keep(t)).map(Item::Track).collect();
        tracks.sort_by_key(|item| match item {
            Item::Track(t) => (t.disc, t.index),
            _ => (0, 0),
        });
        tracks
    }

    pub fn playlist_tracks(&self, playlist: &Playlist) -> Vec<Item<'_>> {
        playlist
            .entries
            .iter()
            .filter_map(|entry| self.tracks.get(&entry.track_id))
            .map(Item::Track)
            .collect()
    }

    fn matches(
        &self,
        item: &Item<'_>,
        types: &[&str],
        parent: Option<Uuid>,
        search: Option<&str>,
        favorites_only: bool,
        artist_ids: &[Uuid],
        album_artist_ids: &[Uuid],
        contributing_artist_ids: &[Uuid],
        album_ids: &[Uuid],
    ) -> bool {
        if matches!(item, Item::Library(_)) && parent.is_some() {
            return false;
        }
        if !types.is_empty() && !types.contains(&item.jellyfin_type()) {
            return false;
        }
        // Derived albums/artists exist to decorate tracks; only the ones the
        // user explicitly saved or followed are part of the library.
        let listed = match item {
            Item::Album(album) => {
                self.saved_albums.contains(&album.id) || !artist_ids.is_empty() || !album_artist_ids.is_empty()
            }
            Item::Artist(artist) => self.followed_artists.contains(&artist.id),
            Item::Track(track) => {
                !self.remote_tracks.contains(&track.id)
                    || search.is_some()
                    || parent.is_some()
                    || !artist_ids.is_empty()
                    || !album_artist_ids.is_empty()
            }
            Item::Library(_) | Item::PlaylistLibrary | Item::Playlist(_) => true,
        };
        if !listed {
            return false;
        }
        let artist_scoped = !artist_ids.is_empty() || !album_artist_ids.is_empty();
        if favorites_only && !artist_scoped && matches!(item, Item::Track(_) | Item::Album(_)) && !self.is_favorite(item.id()) {
            return false;
        }
        if !artist_ids.is_empty() && !self.has_artist(item, artist_ids) {
            return false;
        }
        if !album_artist_ids.is_empty() && !self.has_album_artist(item, album_artist_ids) {
            return false;
        }
        if !contributing_artist_ids.is_empty() && !self.has_contributing_artist(item, contributing_artist_ids) {
            return false;
        }
        if !album_ids.is_empty() && !matches!(item, Item::Track(track) if track.album_id.is_some_and(|id| album_ids.contains(&id))) {
            return false;
        }
        match search {
            Some(term) => self.searchable_text(item).contains_all_words(term),
            None => true,
        }
    }

    fn has_artist(&self, item: &Item<'_>, artist_ids: &[Uuid]) -> bool {
        match item {
            Item::Track(track) => track.artist_ids.iter().any(|id| artist_ids.contains(id)),
            Item::Album(album) => album.artist_ids.iter().any(|id| artist_ids.contains(id)),
            _ => false,
        }
    }

    fn has_album_artist(&self, item: &Item<'_>, artist_ids: &[Uuid]) -> bool {
        match item {
            Item::Track(track) => track
                .album_id
                .and_then(|id| self.albums.get(&id))
                .is_some_and(|album| album.artist_ids.iter().any(|id| artist_ids.contains(id))),
            Item::Album(album) => album.artist_ids.iter().any(|id| artist_ids.contains(id)),
            _ => false,
        }
    }

    fn has_contributing_artist(&self, item: &Item<'_>, artist_ids: &[Uuid]) -> bool {
        match item {
            Item::Track(track) => track.artist_ids.iter().any(|id| artist_ids.contains(id)),
            Item::Album(album) => self.tracks.values().any(|track| {
                track.album_id == Some(album.id)
                    && track.artist_ids.iter().any(|id| artist_ids.contains(id))
                    && !album.artist_ids.iter().any(|id| artist_ids.contains(id))
            }),
            _ => false,
        }
    }

    fn searchable_text(&self, item: &Item<'_>) -> String {
        let mut text = item.name().to_lowercase();
        if let Item::Track(track) = item {
            if let Some(album) = track.album_id.and_then(|id| self.albums.get(&id)) {
                text.push(' ');
                text.push_str(&album.name.to_lowercase());
            }
            for artist in &track.artist_ids {
                if let Some(artist) = self.artists.get(artist) {
                    text.push(' ');
                    text.push_str(&artist.name.to_lowercase());
                }
            }
        }
        text
    }

    pub fn random_tracks(&self, count: usize) -> Vec<Item<'_>> {
        use std::hash::{BuildHasher, Hasher, RandomState};
        let mut hasher = RandomState::new().build_hasher();
        let mut picked: Vec<_> = self.tracks.values().map(Item::Track).collect();
        // Deterministic-enough shuffle without pulling a rand dependency.
        for index in (1..picked.len()).rev() {
            hasher.write_u64(index as u64);
            let swap = (hasher.finish() % (index + 1) as u64) as usize;
            picked.swap(index, swap);
        }
        picked.truncate(count);
        picked
    }
}
