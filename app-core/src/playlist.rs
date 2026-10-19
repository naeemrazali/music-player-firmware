use heapless::Deque;

use crate::track::Track;

const MAX_TRACKS: usize = 16;

pub enum SortOrder {
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

    pub fn add(&mut self, track: Track) -> Result<(), ()> {
        self.tracks.push_back(track).map_err(|_| ())
    }

    pub fn current(&self) -> Option<&Track> {
        self.tracks.get(self.current_index)
    }

    pub fn next(&mut self) -> Option<&Track> {
        if self.current_index + 1 < self.tracks.len() {
            self.current_index += 1;
            self.current()
        } else {
            None
        }
    }

    pub fn prev(&mut self) -> Option<&Track> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.current()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
