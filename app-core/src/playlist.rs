use heapless::Deque;

use crate::track::Track;

const MAX_TRACKS: usize = 16;

#[derive(Default, Clone, Copy)]
pub enum SortOrder {
    #[default]
    TitleAscending,
    TitleDescending,
    ArtistAscending,
    ArtistDescending,
}

#[derive(Default, Clone)]
pub struct Playlist {
    tracks: Deque<Track, MAX_TRACKS>,
    current_index: usize,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn populate(&mut self, tracks: &Deque<Track, MAX_TRACKS>) {
        self.tracks.clear();
        self.tracks.clone_from(tracks);
    }

    pub fn add_back(&mut self, track: Track) {
        if self.tracks.is_full() {
            self.tracks.pop_front();
        }
        // TODO(errors) handle the result
        let _ = self.tracks.push_back(track);
    }

    pub fn add_front(&mut self, track: Track) {
        if self.tracks.is_full() {
            self.tracks.pop_back();
        }
        // TODO(errors) handle result
        let _ = self.tracks.push_front(track);
    }

    pub fn current(&self) -> Option<Track> {
        self.tracks.get(self.current_index).cloned()
    }

    pub fn prev(&mut self) -> Option<Track> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.current()
        } else {
            None
        }
    }
}

impl Iterator for Playlist {
    type Item = Track;

    // TODO(feature) wrap around once actual playlist is empty
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index + 1 < self.tracks.len() {
            self.current_index += 1;
            self.current()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
