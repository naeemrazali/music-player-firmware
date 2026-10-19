use heapless::Deque;

use crate::track::Track;

pub struct Playlist {
    tracks: Deque<Track, 16>,
    current_index: usize,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            tracks: Deque::new(),
            current_index: 0,
        }
    }
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn populate(&mut self, tracks: &Deque<Track, 16>) {
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
