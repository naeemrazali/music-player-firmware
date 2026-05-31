use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

use crate::ui;

pub enum AppScreen<'a> {
    Main(&'a mut MainScreen),
    Settings(&'a mut SettingsScreen),
}

impl AppScreen<'_> {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        match self {
            AppScreen::Main(s) => s.draw(display),
            AppScreen::Settings(s) => s.draw(display),
        }
    }
}

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
            play_button: (ui::PlayButton::new(Point::new(120, 170), 12, Gray8::BLACK, true)),
        }
    }
}

pub struct SettingsScreen {
    pub background: Gray8,
    pub heading: ui::Label<'static>,
    pub separator: ui::HorizontalLine,
    pub list: ui::List<'static>,
}

impl SettingsScreen {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        self.heading.draw(display);
        self.separator.draw(display);
        self.list.draw(display);
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self {
            background: Gray8::new(0x9F),
            heading: ui::Label::new(
                "Settings",
                Point::new(10, 20),
                MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
            ),
            separator: ui::HorizontalLine::new(
                Point::new(10, 34),
                Point::new(230, 34),
                PrimitiveStyle::with_stroke(Gray8::new(0x64), 1),
            ),
            list: ui::List {
                items: &["Volume", "Shuffle", "Repeat", "Equalizer"],
                selected: 0,
                start: Point::new(10, 46),
                item_height: 14,
                selected_color: Gray8::BLACK,
                unselected_color: Gray8::new(100),
                selected_prefix: "> ",
                unselected_prefix: "  ",
            },
        }
    }
}
