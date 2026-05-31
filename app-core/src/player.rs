pub enum PlaybackState {
    Playing,
    Paused,
}

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

    /// Toggle between Playing and Paused.
    /// Called by the event handler when the play/pause button is pressed.
    pub fn toggle_playback(&mut self) {
        self.state = match self.state {
            PlaybackState::Playing => PlaybackState::Paused,
            PlaybackState::Paused => PlaybackState::Playing,
        };
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
    }

    /// Advance elapsed time by `delta_ms`, but only if currently playing.
    /// Called by the update loop every frame.
    pub fn tick(&mut self, delta_ms: u32) {
        if self.is_playing() {
            self.elapsed_ms += delta_ms;
            if self.elapsed_ms > self.total_ms {
                self.elapsed_ms = self.total_ms;
            }
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
