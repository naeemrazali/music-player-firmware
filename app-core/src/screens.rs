use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

use crate::ui;

pub enum AppScreen {
    Main(MainScreen),
    Settings(SettingsScreen),
}

impl AppScreen {
    pub fn draw(
        &mut self,
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
    pub title: &'static str,
    pub artist: &'static str,
    pub elapsed_ms: u32,
    pub total_ms: u32,
    pub is_playing: bool,
    pub volume: u8,
    pub elapsed_time_buf: [u8; 6],
    pub total_time_buf: [u8; 6],
}

impl MainScreen {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        ui::draw_text(display, self.title, Point::new(10, 20), Gray8::BLACK);
        ui::draw_text(display, self.artist, Point::new(10, 36), Gray8::new(100));
        ui::draw_progress_bar(
            display,
            self.total_ms,
            self.elapsed_ms,
            Rectangle::new(Point::new(10, 100), Size::new(220, 6)),
            Gray8::new(0),
            Gray8::new(160),
        );
        ui::draw_text(
            display,
            ui::fmt_time_ms(self.elapsed_ms, &mut self.elapsed_time_buf),
            Point::new(10, 112),
            Gray8::new(80),
        );
        ui::draw_text(
            display,
            ui::fmt_time_ms(self.total_ms, &mut self.total_time_buf),
            Point::new(200, 112),
            Gray8::new(80),
        );
        ui::draw_play_button(
            display,
            Point::new(120, 170),
            self.is_playing,
            Gray8::new(0),
        );
    }
}

pub struct SettingsScreen {
    pub background: Gray8,
    pub items: &'static [&'static str],
    pub selected: usize,
}

impl SettingsScreen {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);

        // Title
        ui::draw_text(display, "Settings", Point::new(10, 20), Gray8::new(0));

        let hline = Point::new(10, 34);
        let hline_end = Point::new(230, 34);
        use embedded_graphics::primitives::Line;
        Line::new(hline, hline_end)
            .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(
                Gray8::new(100),
                1,
            ))
            .draw(display)
            .unwrap();

        // List items
        for (i, item) in self.items.iter().enumerate() {
            let y = 46 + i as i32 * 14;
            let color = if i == self.selected {
                Gray8::BLACK
            } else {
                Gray8::new(100)
            };
            let prefix = if i == self.selected { "> " } else { "  " };
            ui::draw_text(display, prefix, Point::new(10, y), color);
            ui::draw_text(display, item, Point::new(22, y), color);
        }
    }
}
