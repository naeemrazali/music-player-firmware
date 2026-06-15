#[derive(Copy, Clone)]
pub enum Button {
    Play,
    Menu,
    Next,
    Prev,
}

pub enum Playback {
    Play,
    Pause,
    Seek(u32),
    NextTrack,
    PreviousTrack,
    Stop,
}

pub enum Event {
    ButtonPress(Button),
    Player(Playback),
}
