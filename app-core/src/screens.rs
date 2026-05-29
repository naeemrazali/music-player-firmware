use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
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

        let title_label = ui::Label {
            text:     self.title,
            position: Point::new(10, 20),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
        };
        ui::draw_text(display, &title_label);

        let artist_label = ui::Label {
            text:     self.artist,
            position: Point::new(10, 36),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(100)),
        };
        ui::draw_text(display, &artist_label);

        ui::draw_progress_bar(
            display,
            self.total_ms,
            self.elapsed_ms,
            Rectangle::new(Point::new(10, 100), Size::new(220, 6)),
            Gray8::new(0),
            Gray8::new(160),
        );

        let elapsed_str = ui::fmt_time_ms(self.elapsed_ms, &mut self.elapsed_time_buf);
        let elapsed_label = ui::Label {
            text:     elapsed_str,
            position: Point::new(10, 112),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(80)),
        };
        ui::draw_text(display, &elapsed_label);

        let total_str = ui::fmt_time_ms(self.total_ms, &mut self.total_time_buf);
        let total_label = ui::Label {
            text:     total_str,
            position: Point::new(200, 112),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(80)),
        };
        ui::draw_text(display, &total_label);

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
        let settings_label = ui::Label {
            text:     "Settings",
            position: Point::new(10, 20),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(0)),
        };
        ui::draw_text(display, &settings_label);
        ui::draw_line(
            display,
            Point::new(10, 34),
            Point::new(230, 34),
            PrimitiveStyle::with_stroke(Gray8::new(100), 1),
        );

        let list = ui::List {
            items:            self.items,
            selected:         self.selected,
            start:            Point::new(10, 46),
            item_height:      14,
            selected_color:   Gray8::BLACK,
            unselected_color: Gray8::new(100),
            selected_prefix:  "> ",
            unselected_prefix:"  ",
        };
        ui::draw_list(display, &list);
    }
}
