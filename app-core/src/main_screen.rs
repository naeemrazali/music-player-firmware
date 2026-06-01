use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

use crate::playback_state::PlaybackState;
use crate::ui;

pub struct MainScreen {
    pub background: Gray8,
    pub title: ui::Label<'static>,
    pub artist: ui::Label<'static>,
    pub elapsed_time: ui::Label<'static>,
    pub total_time: ui::Label<'static>,
    pub progress: ui::ProgressBar,
    pub play_button: ui::PlayButton,
}

impl MainScreen {
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
}

impl Default for MainScreen {
    fn default() -> Self {
        Self {
            background: (Gray8::new(0x9F)),
            title: (ui::Label::new(
                "Clair de Lune",
                Point::new(10, 20),
                MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
            )),
            artist: (ui::Label::new(
                "Claude Debussy",
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
            play_button: (ui::PlayButton::new(
                Point::new(120, 170),
                12,
                Gray8::BLACK,
                PlaybackState::Playing,
            )),
        }
    }
}
