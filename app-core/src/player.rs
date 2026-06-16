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
        Self {
            is_playing: false,
            elapsed_ms: 0,
            playlist,
            events: heapless::Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Player(Playback::Seek(percent)) => self.seek_to(*percent),
            Event::Player(Playback::NextTrack) => self.next_track(),
            Event::Player(Playback::PreviousTrack) => self.prev_track(),
            Event::Player(Playback::Stop) => self.stop(),
            Event::Player(Playback::Toggle) => self.toggle_playback(),
            _ => (),
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
    }

    fn pause(&mut self) {
        self.is_playing = false;
    }

    fn stop(&mut self) {
        self.pause();
        self.playlist = Playlist::new();
        self.add_event(Event::Player(Playback::Stopped));
    }

    fn toggle_playback(&mut self) {
        match self.is_playing {
            true => self.pause(),
            false => self.play(),
        };
        self.add_event(Event::Player(Playback::Toggled(self.is_playing)));
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
                self.advance_track_or_stop();
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

    fn advance_track_or_stop(&mut self) {
        if self.playlist.next().is_none() {
            self.stop();
        } else {
            self.next_track();
        }
    }

    fn total_ms(&self) -> u32 {
        self.playlist.current().map(|t| t.duration_ms).unwrap_or(0)
    }

    fn seek_to(&mut self, percent: u32) {
        let total = self.total_ms();
        let elapsed_ms = (percent * total).min(total);

        self.elapsed_ms = elapsed_ms;
        self.add_event(Event::Player(Playback::ProgressUpdated {
            elapsed_ms,
            percent_elapsed: percent,
        }));
        if elapsed_ms == total && total > 0 {
            self.advance_track_or_stop();
        }
    }

    fn next_track(&mut self) {
        let track = self.playlist.next().copied();
        self.elapsed_ms = 0;
        self.add_event(Event::Player(Playback::TrackChanged(track)));
    }

    fn prev_track(&mut self) {
        let track = self.playlist.prev().copied();
        self.elapsed_ms = 0;
        self.add_event(Event::Player(Playback::TrackChanged(track)));
    }
}

#[cfg(test)]
mod tests;
