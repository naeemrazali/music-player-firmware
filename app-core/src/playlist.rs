use heapless::Vec;

use crate::track::Track;

pub struct Playlist {
    tracks: Vec<Track, 32>,
    current_index: usize,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add(&mut self, track: Track) -> Result<(), ()> {
        self.tracks.push(track).map_err(|_| ())
    }

    pub fn current(&self) -> Option<&Track> {
        self.tracks.get(self.current_index)
    }

    pub fn next(&mut self) -> Option<&Track> {
        if self.current_index + 1 < self.tracks.len() {
            self.current_index += 1;
        }
        self.current()
    }

    pub fn prev(&mut self) -> Option<&Track> {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
        self.current()
    }
}
