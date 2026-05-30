use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;

use crate::ui;

pub enum AppScreen<'a> {
    Main(&'a mut MainScreen),
    Settings(&'a mut SettingsScreen),
}

impl AppScreen<'_> {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        match self {
            AppScreen::Main(s)     => s.draw(display),
            AppScreen::Settings(s) => s.draw(display),
        }
    }
}

pub struct MainScreen {
    pub background: Gray8,
    pub title:      ui::Widget<'static>,
    pub artist:     ui::Widget<'static>,
    pub progress:   ui::Widget<'static>,
    pub play_btn:   ui::Widget<'static>,
    pub elapsed_buf: [u8; 6],
    pub total_buf:   [u8; 6],
}

impl MainScreen {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);

        for widget in [&self.title, &self.artist, &self.progress, &self.play_btn] {
            widget.draw(display);
        }

        let (elapsed_ms, total_ms) = if let ui::Widget::ProgressBar(ref pb) = self.progress {
            (pb.elapsed_ms, pb.total_ms)
        } else {
            (0, 1)
        };

        let elapsed_str = ui::fmt_time_ms(elapsed_ms, &mut self.elapsed_buf);
        ui::Label {
            text:     elapsed_str,
            position: Point::new(10, 112),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(80)),
        }
        .draw(display);

        let total_str = ui::fmt_time_ms(total_ms, &mut self.total_buf);
        ui::Label {
            text:     total_str,
            position: Point::new(200, 112),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(80)),
        }
        .draw(display);
    }
}

pub struct SettingsScreen {
    pub background: Gray8,
    pub heading:    ui::Widget<'static>,
    pub separator:  ui::Widget<'static>,
    pub list:       ui::Widget<'static>,
}

impl SettingsScreen {
    pub fn draw(
        &mut self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);

        for widget in [&self.heading, &self.separator, &self.list] {
            widget.draw(display);
        }
    }
}
