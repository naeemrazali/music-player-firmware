use crate::event::{Command, Event, State};
use crate::playlist::Playlist;

pub struct Player {
    is_playing: bool,
    elapsed_ms: u32,
    playlist: Playlist,
    events: heapless::Vec<Event, 8>,
}

impl Player {
    pub fn new(playlist: Playlist) -> Self {
        let mut player = Self {
            is_playing: true,
            elapsed_ms: 0,
            playlist,
            events: heapless::Vec::new(),
        };
        player.current_track();
        player.play();
        player.seek_to(0);
        player
    }

    pub fn handle_event(&mut self, event: &Event) -> heapless::Vec<Event, 8> {
        match event {
            Event::Player(Command::Seek(percent)) => self.seek_to(*percent),
            Event::Player(Command::NextTrack) => self.next_track(),
            Event::Player(Command::PreviousTrack) => self.prev_track(),
            Event::Player(Command::Stop) => self.stop(),
            Event::Player(Command::Toggle) => self.toggle_playback(),
            Event::Player(Command::Tick(time)) => self.tick(*time),
            _ => (),
        }
        self.push_events()
    }

    pub fn push_events(&mut self) -> heapless::Vec<Event, 8> {
        let mut events = heapless::Vec::new();
        core::mem::swap(&mut self.events, &mut events);
        events
    }

    fn add_event(&mut self, event: Event) {
        let _ = self.events.push(event);
    }

    fn play(&mut self) {
        self.is_playing = true;
        self.add_event(Event::Playback(State::Toggled(self.is_playing)));
    }

    fn pause(&mut self) {
        self.is_playing = false;
        self.add_event(Event::Playback(State::Toggled(self.is_playing)));
    }

    fn stop(&mut self) {
        self.pause();
        self.add_event(Event::Playback(State::Stopped));
    }

    fn toggle_playback(&mut self) {
        match self.is_playing {
            true => self.pause(),
            false => self.play(),
        };
    }

    pub fn tick(&mut self, delta_ms: u32) {
        if self.is_playing {
            let total = self.total_ms();
            self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms).min(total);
            self.add_event(Event::Playback(State::ProgressUpdated {
                elapsed_ms: self.elapsed_ms,
                percent_elapsed: self.percent_elapsed(),
            }));
            if self.elapsed_ms == total && total > 0 {
                self.next_track();
            }
        }
    }

    fn percent_elapsed(&self) -> u32 {
        let total = self.total_ms();
        if total == 0 {
            return 0;
        }
        (self.elapsed_ms * 100) / total
    }

    fn total_ms(&self) -> u32 {
        self.playlist.current().map(|t| t.duration_ms).unwrap_or(0)
    }

    fn seek_to(&mut self, percent: u32) {
        let total = self.total_ms();
        let elapsed_ms = ((percent * total) / 100).min(total);

        self.elapsed_ms = elapsed_ms;
        self.add_event(Event::Playback(State::ProgressUpdated {
            elapsed_ms,
            percent_elapsed: percent,
        }));
        if elapsed_ms == total && total > 0 {
            self.next_track();
        }
    }

    fn current_track(&mut self) {
        let track = self.playlist.current().copied();
        self.add_event(Event::Playback(State::TrackChanged(track)));
    }

    fn next_track(&mut self) {
        let track = self.playlist.next().copied();
        match track {
            Some(_) => {
                self.elapsed_ms = 0;
                self.add_event(Event::Playback(State::TrackChanged(track)));
            }
            None => {
                self.stop();
            }
        }
    }

    fn prev_track(&mut self) {
        let track = self.playlist.prev().copied();
        match track {
            Some(_) => {
                self.elapsed_ms = 0;
                self.add_event(Event::Playback(State::TrackChanged(track)));
            }
            None => {
                self.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests;
