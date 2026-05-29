pub enum PlaybackState {
    Playing,
    Paused,
}

pub struct Player {
    pub state: PlaybackState,
    pub elapsed_ms: u32,
}
