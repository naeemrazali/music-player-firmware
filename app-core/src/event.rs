use crate::{track::Track, ui::screen_names::ScreenName};

#[derive(Clone, Copy)]
pub enum Button {
    Play,
    Menu,
    Next,
    Prev,
    Seek(u32),
}

#[derive(Clone, Copy)]
pub enum Command {
    Toggle,
    Stopped,
    Seek(u32),
    NextTrack,
    PreviousTrack,
    Stop,
    Tick(u32),
}

#[derive(Clone, Copy)]
pub enum State {
    TrackChanged(Option<Track>),
    Toggled(bool),
    ProgressUpdated {
        elapsed_ms: u32,
        percent_elapsed: u32,
    },
}

#[derive(Clone, Copy)]
pub enum Screen {
    Refresh,
    Change(ScreenName),
}

#[derive(Clone, Copy)]
pub enum Event {
    ButtonPress(Button),
    Player(Command),
    Playback(State),
    Ui(Screen),
}
