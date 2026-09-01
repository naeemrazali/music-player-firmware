use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

use crate::event::{Button, Event, Playback, Screen};
use crate::track::Track;
use crate::ui;
use crate::ui::screen_names::ScreenName;

pub struct MainScreen {
    background: Gray8,
    title: ui::Label<'static>,
    artist: ui::Label<'static>,
    elapsed_time: ui::Label<'static>,
    total_time: ui::Label<'static>,
    progress: ui::ProgressBar,
    play_button: ui::PlayButton,
    events: heapless::Vec<Event, 8>,
}

impl MainScreen {
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::ButtonPress(Button::Play) => {
                self.add_event(Event::Player(Playback::Toggle));
            }
            Event::ButtonPress(Button::Menu) => {
                self.add_event(Event::Ui(Screen::Change(ScreenName::Settings)));
                self.add_event(Event::Ui(Screen::Refresh));
            }
            Event::ButtonPress(Button::Next) => {
                self.add_event(Event::Player(Playback::NextTrack));
            }
            Event::ButtonPress(Button::Prev) => {
                self.add_event(Event::Player(Playback::PreviousTrack));
            }
            Event::ButtonPress(Button::Seek(percent)) => {
                self.add_event(Event::Player(Playback::Seek(*percent)));
            }
            Event::Player(Playback::TrackChanged(track)) => {
                self.sync_new_track(track);
                self.add_event(Event::Ui(Screen::Refresh));
            }
            Event::Player(Playback::Toggled(is_playing)) => {
                self.update_play_button(*is_playing);
                self.add_event(Event::Ui(Screen::Refresh));
            }
            Event::Player(Playback::ProgressUpdated {
                elapsed_ms,
                percent_elapsed,
            }) => {
                self.update_progress_bar(*elapsed_ms, *percent_elapsed);
                self.add_event(Event::Ui(Screen::Refresh));
            }
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

    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        self.title.draw(display);
        self.artist.draw(display);
        self.elapsed_time.draw(display);
        self.total_time.draw(display);
        self.progress.draw(display);
        self.play_button.draw(display);
    }

    fn sync_new_track(&mut self, track: &Option<Track>) {
        match track {
            Some(track) => {
                self.title.set_text(track.title);
                self.artist.set_text(track.artist);
                self.elapsed_time.set_text("00:00");
                self.total_time
                    .set_text(core::str::from_utf8(&ui::fmt_time_ms(track.duration_ms)).unwrap());
            }
            None => {
                self.title.set_text(Track::default().title);
                self.artist.set_text(Track::default().artist);
                self.elapsed_time.set_text("00:00");
                self.total_time.set_text("00:00");
            }
        }
    }

    fn update_progress_bar(&mut self, elapsed_ms: u32, percent_elapsed: u32) {
        self.progress.percent = percent_elapsed;
        self.elapsed_time
            .set_text(core::str::from_utf8(&ui::fmt_time_ms(elapsed_ms)).unwrap());
    }

    fn update_play_button(&mut self, is_playing: bool) {
        self.play_button.is_playing = is_playing;
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self {
            background: (Gray8::new(0x9F)),
            title: (ui::Label::new(
                "",
                Point::new(10, 20),
                MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
            )),
            artist: (ui::Label::new(
                "",
                Point::new(10, 36),
                MonoTextStyle::new(&FONT_6X10, Gray8::new(0x64)),
            )),
            elapsed_time: (ui::Label::new(
                "00:00",
                Point::new(10, 112),
                MonoTextStyle::new(&FONT_6X10, Gray8::new(0x64)),
            )),
            total_time: (ui::Label::new(
                "03:00",
                Point::new(200, 112),
                MonoTextStyle::new(&FONT_6X10, Gray8::new(0x64)),
            )),
            progress: (ui::ProgressBar::new(
                Rectangle::new(Point::new(10, 100), Size::new(220, 6)),
                50,
                Gray8::new(0xA0),
                Gray8::BLACK,
            )),
            play_button: (ui::PlayButton::new(Point::new(120, 170), 12, Gray8::BLACK, false)),
            events: heapless::Vec::new(),
        }
    }
}
