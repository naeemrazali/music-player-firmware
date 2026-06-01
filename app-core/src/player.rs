pub mod playback_state;

use playback_state::PlaybackState;

pub struct Player {
    state: PlaybackState,
    elapsed_ms: u32,
    total_ms: u32,
}

impl Player {
    pub fn new(total_ms: u32) -> Self {
        Self {
            state: PlaybackState::Playing,
            elapsed_ms: 0,
            total_ms,
        }
    }

    pub fn toggle_playback(&mut self) -> PlaybackState {
        self.state = match self.state {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused => PlaybackState::Playing,
        };
        self.state
    }

    pub fn tick(&mut self, delta_ms: u32) {
        if matches!(self.state, PlaybackState::Playing) {
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
}
