use crate::event::{Event, Playback};
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
        player.initialize();
        player
    }

    pub fn handle_event_queue(&mut self, event_queue: &mut heapless::Vec<Event, 8>) {
        event_queue.retain(|event| self.handle_event(event).is_some());
    }

    fn handle_event(&mut self, event: &Event) -> Option<Event> {
        let mut ret = None;
        match event {
            Event::Player(Playback::Seek(percent)) => self.seek_to(*percent),
            Event::Player(Playback::NextTrack) => self.next_track(),
            Event::Player(Playback::PreviousTrack) => self.prev_track(),
            Event::Player(Playback::Stop) => self.stop(),
            Event::Player(Playback::Toggle) => self.toggle_playback(),
            Event::Player(Playback::Tick(time)) => self.tick(*time),
            _ => ret = Some(*event),
        }
        ret
    }

    fn initialize(&mut self) {
        let track = self.playlist.current().copied();
        self.add_event(Event::Player(Playback::TrackChanged(track)));
        self.add_event(Event::Player(Playback::Toggled(self.is_playing)));
        if track.is_some() {
            self.add_event(Event::Player(Playback::ProgressUpdated {
                elapsed_ms: self.elapsed_ms,
                percent_elapsed: self.percent_elapsed(),
            }));
        }
    }

    pub fn event_queue(&mut self) -> heapless::Vec<Event, 8> {
        let mut queue = heapless::Vec::new();
        core::mem::swap(&mut self.events, &mut queue);
        queue
    }

    fn add_event(&mut self, event: Event) {
        let _ = self.events.push(event);
    }

    fn play(&mut self) {
        self.is_playing = true;
        self.add_event(Event::Player(Playback::Toggled(self.is_playing)));
    }

    fn pause(&mut self) {
        self.is_playing = false;
        self.add_event(Event::Player(Playback::Toggled(self.is_playing)));
    }

    fn stop(&mut self) {
        self.pause();
        self.add_event(Event::Player(Playback::Stopped));
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
            self.add_event(Event::Player(Playback::ProgressUpdated {
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
        self.add_event(Event::Player(Playback::ProgressUpdated {
            elapsed_ms,
            percent_elapsed: percent,
        }));
        if elapsed_ms == total && total > 0 {
            self.next_track();
        }
    }

    fn next_track(&mut self) {
        let track = self.playlist.next().copied();
        match track {
            Some(_) => {
                self.elapsed_ms = 0;
                self.add_event(Event::Player(Playback::TrackChanged(track)));
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
                self.add_event(Event::Player(Playback::TrackChanged(track)));
            }
            None => {
                self.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests;
