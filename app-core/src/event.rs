use crate::{track::Track, ui::screen_names::ScreenName};

const QUEUE_LENGTH: usize = 8;
type EventQueue = heapless::Vec<Event, QUEUE_LENGTH>;

pub trait EventHandler {
    fn event_queue(&mut self) -> &mut EventQueue;
    fn handle_event(&mut self, event: &Event) -> EventQueue;
    fn push_events(&mut self) -> EventQueue {
        let mut events = heapless::Vec::new();
        core::mem::swap(self.event_queue(), &mut events);
        events
    }
    fn add_event(&mut self, event: Event) {
        let _ = self.event_queue().push(event);
    }
}

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
    Stopped,
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
