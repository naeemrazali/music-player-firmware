use crate::track::Track;

pub enum PlaybackState {
    Playing,
    Paused,
}

pub struct Player {
    state: PlaybackState,
    elapsed_ms: u32,
    total_ms: u32,
    current_track: Track,
}

impl Player {
    pub fn new(total_ms: u32) -> Self {
        Self {
            state: PlaybackState::Playing,
            elapsed_ms: 0,
            total_ms,
            current_track: Track {
                title:       "Clair de Lune",
                artist:      "Claude Debussy",
                duration_ms: total_ms,
                file_path:   "/music/flac/clair_de_lune.flac",
            },
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
            self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms).min(self.total_ms);
        }
    }

    pub fn elapsed_ms(&self) -> u32 {
        self.elapsed_ms
    }

    pub fn total_ms(&self) -> u32 {
        self.total_ms
    }

    pub fn progress_percent(&self) -> u32 {
        (self.elapsed_ms * 100) / self.total_ms
    }

    pub fn current_track(&self) -> Option<&Track> {
        Some(&self.current_track)
    }
}
