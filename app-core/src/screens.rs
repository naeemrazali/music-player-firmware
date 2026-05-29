use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

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
    pub title_label: ui::Label<'static>,
    pub artist_label: ui::Label<'static>,
    pub elapsed_ms: u32,
    pub total_ms: u32,
    pub is_playing: bool,
    pub volume: u8,
    pub elapsed_time_buf: [u8; 6],
    pub total_time_buf: [u8; 6],
    pub default_text_style: MonoTextStyle<'static, Gray8>,
}

impl MainScreen {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        ui::draw_label(display, &self.title_label);
        ui::draw_label(display, &self.artist_label);
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
            self.default_text_style,
        );
        ui::draw_text(
            display,
            ui::fmt_time_ms(self.total_ms, &mut self.total_time_buf),
            Point::new(200, 112),
            self.default_text_style,
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
    pub heading: ui::Label<'static>,
    pub items: &'static [&'static str],
    pub selected: usize,
}

impl SettingsScreen {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        ui::draw_label(display, &self.heading);
        ui::draw_line(
            display,
            Point::new(10, 34),
            Point::new(230, 34),
            PrimitiveStyle::with_stroke(Gray8::new(100), 1),
        );

        let list = ui::List {
            items: self.items,
            selected: self.selected,
            start: Point::new(10, 46),
            item_height: 14,
            selected_color: Gray8::BLACK,
            unselected_color: Gray8::new(100),
            selected_prefix: "> ",
            unselected_prefix: "  ",
        };
        ui::draw_list(display, &list);
    }
}
