use crate::playlist::Playlist;
use crate::track::Track;

pub enum PlaybackState {
    Playing,
    Paused,
}

pub struct Player {
    state: PlaybackState,
    elapsed_ms: u32,
    playlist: Playlist,
}

impl Player {
    pub fn new(playlist: Playlist) -> Self {
        Self {
            state: PlaybackState::Playing,
            elapsed_ms: 0,
            playlist,
        }
    }

    pub fn toggle_playback(&mut self) {
        self.state = match self.state {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused => PlaybackState::Playing,
        };
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
    }

    pub fn tick(&mut self, delta_ms: u32) {
        if self.is_playing() {
            let total = self.total_ms();
            self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms).min(total);
        }
    }

    pub fn elapsed_ms(&self) -> u32 {
        self.elapsed_ms
    }

    pub fn total_ms(&self) -> u32 {
        self.playlist.current().map(|t| t.duration_ms).unwrap_or(0)
    }

    pub fn progress_percent(&self) -> u32 {
        let total = self.total_ms();
        if total == 0 {
            return 0;
        }
        (self.elapsed_ms * 100) / total
    }

    pub fn seek_to(&mut self, ms: u32) {
        self.elapsed_ms = ms.min(self.total_ms());
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.playlist.current()
    }

    pub fn next_track(&mut self) {
        let _ = self.playlist.next();
        self.elapsed_ms = 0;
    }

    pub fn prev_track(&mut self) {
        let _ = self.playlist.prev();
        self.elapsed_ms = 0;
    }
}
