use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;

// Shared toolkit
use crate::ui;

pub struct MainScreen {
    pub background: Gray8,
    pub title: &'static str,
    pub artist: &'static str,
}

impl MainScreen {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        ui::draw_text(display, self.title, Point::new(10, 20), Gray8::BLACK);
        ui::draw_text(display, self.artist, Point::new(10, 36), Gray8::new(64));
    }
}

pub struct SettingsScreen {
    pub background: Gray8,
    pub label: &'static str,
}

impl SettingsScreen {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        ui::draw_text(display, self.label, Point::new(10, 20), Gray8::BLACK);
    }
}

pub enum AppScreen {
    Main(MainScreen),
    Settings(SettingsScreen),
}

impl AppScreen {
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
