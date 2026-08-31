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
    pub name: String,
}

#[derive(Clone)]
pub struct Album {
    pub id: Uuid,
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
    pub image: Option<String>,
    /// Spotify URI for client-backed playlists; None for ephemeral ones.
    pub spotify_uri: Option<String>,
    pub entries: Vec<PlaylistEntry>,
}

/// Unified read-only view over every kind of catalog item, used by the
/// query engine and DTO builder.
#[derive(Clone, Copy)]
pub enum Item<'a> {
    Library(&'a str),
    Track(&'a Track),
    Album(&'a Album),
    Artist(&'a Artist),
    Playlist(&'a Playlist),
}

impl Item<'_> {
    pub fn id(&self) -> Uuid {
        match self {
            Item::Library(_) => library_id(),
            Item::Track(t) => t.id,
            Item::Album(a) => a.id,
            Item::Artist(a) => a.id,
            Item::Playlist(p) => p.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Item::Library(name) => name,
            Item::Track(t) => &t.name,
            Item::Album(a) => &a.name,
            Item::Artist(a) => &a.name,
            Item::Playlist(p) => &p.name,
        }
    }

    pub fn jellyfin_type(&self) -> &'static str {
        match self {
            Item::Library(_) => "CollectionFolder",
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
    favorites: HashSet<Uuid>,
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
        fresh.favorites = std::mem::take(&mut self.favorites);
        fresh.play_counts = std::mem::take(&mut self.play_counts);
        fresh.positions = std::mem::take(&mut self.positions);
        fresh.last_played = std::mem::take(&mut self.last_played);
        for (id, track) in std::mem::take(&mut self.tracks) {
            fresh.tracks.entry(id).or_insert(track);
        }
        for (id, album) in std::mem::take(&mut self.albums) {
            fresh.albums.entry(id).or_insert(album);
        }
        for (id, artist) in std::mem::take(&mut self.artists) {
            fresh.artists.entry(id).or_insert(artist);
        }
        *self = fresh;
    }

    pub fn item(&self, id: Uuid) -> Option<Item<'_>> {
        if id == library_id() {
            return Some(Item::Library(LIBRARY_NAME));
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
        self.favorites.contains(&id)
    }

    pub fn set_favorite(&mut self, id: Uuid, favorite: bool) {
        if favorite {
            self.favorites.insert(id);
        } else {
            self.favorites.remove(&id);
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

    /// Direct children of `parent`; with the whole catalog when recursive.
    pub fn query(&self, types: &[&str], parent: Option<Uuid>, search: Option<&str>) -> Vec<Item<'_>> {
        let mut items = self.candidate_items(parent);
        items.retain(|item| self.matches(item, types, parent, search));
        items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
        items
    }

    fn candidate_items(&self, parent: Option<Uuid>) -> Vec<Item<'_>> {
        match parent {
            Some(id) if id != library_id() => match self.item(id) {
                Some(Item::Album(_)) => self.track_children(|t| t.album_id == Some(id)),
                Some(Item::Artist(_)) => {
                    let albums = self.albums.values().filter(|a| a.artist_ids.contains(&id));
                    albums.map(Item::Album).collect()
                }
                Some(Item::Playlist(playlist)) => self.playlist_tracks(playlist),
                _ => vec![],
            },
            _ => self.all_items(),
        }
    }

    fn all_items(&self) -> Vec<Item<'_>> {
        let mut items = vec![Item::Library(LIBRARY_NAME)];
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

    fn matches(&self, item: &Item<'_>, types: &[&str], parent: Option<Uuid>, search: Option<&str>) -> bool {
        if matches!(item, Item::Library(_)) && parent.is_some() {
            return false;
        }
        if !types.is_empty() && !types.contains(&item.jellyfin_type()) {
            return false;
        }
        // Derived albums/artists exist to decorate tracks; only the ones the
        // user explicitly saved or followed are part of the library.
        let listed = match item {
            Item::Album(album) => self.saved_albums.contains(&album.id),
            Item::Artist(artist) => self.followed_artists.contains(&artist.id),
            _ => true,
        };
        if !listed {
            return false;
        }
        match search {
            Some(term) => self.searchable_text(item).contains_all_words(term),
            None => true,
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
