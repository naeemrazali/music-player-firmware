use crate::{track::Track, ui::screen_names::ScreenName};

#[derive(Copy, Clone)]
pub enum Button {
    Play,
    Menu,
    Next,
    Prev,
    Seek(u32),
}

pub enum Playback {
    Toggle,
    Stopped,
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
}

pub enum Screen {
    Refresh,
    Change(ScreenName),
}

pub enum Event {
    ButtonPress(Button),
    Player(Playback),
    Ui(Screen),
}
