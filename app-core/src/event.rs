use crate::track::Track;

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
    TrackChanged(Option<Track>),
    Toggled(bool),
    ProgressUpdated {
        elapsed_ms: u32,
        percent_elapsed: u32,
    },
    Toggle,
    Stopped,
}

pub enum Event {
    ButtonPress(Button),
    Player(Playback),
    RefreshScreen,
}
