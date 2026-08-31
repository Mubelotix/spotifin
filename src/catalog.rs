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
}

#[derive(Clone)]
pub struct Track {
    pub id: Uuid,
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
    pub track_id: Uuid,
}

#[derive(Clone)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub image: Option<String>,
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

pub const LIBRARY_NAME: &str = "Music";

#[derive(Default)]
pub struct Catalog {
    pub artists: HashMap<Uuid, Artist>,
    pub albums: HashMap<Uuid, Album>,
    pub tracks: HashMap<Uuid, Track>,
    pub playlists: HashMap<Uuid, Playlist>,
    favorites: HashSet<Uuid>,
    play_counts: HashMap<Uuid, u32>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
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
        if let Some(term) = search {
            if !item.name().to_lowercase().contains(&term.to_lowercase()) {
                return false;
            }
        }
        true
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
