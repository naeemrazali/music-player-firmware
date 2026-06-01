pub mod display_config;
pub mod screen_manager;
pub mod screen_names;
pub mod screens;

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Gray8,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle},
    text::Text,
};
use heapless::String;

use crate::player::playback_state::PlaybackState;

pub fn clear_background(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    color: Gray8,
) {
    display.clear(color).unwrap();
}

pub struct Label<'a> {
    pub text: String<32>,
    pub position: Point,
    pub style: MonoTextStyle<'a, Gray8>,
}

impl<'a> Label<'a> {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        Text::new(&self.text, self.position, self.style)
            .draw(display)
            .unwrap();
    }

    pub fn new(text: &str, position: Point, style: MonoTextStyle<'a, Gray8>) -> Self {
        let mut s = String::<32>::new();
        let _ = s.push_str(text);
        Self {
            text: s,
            position,
            style,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text.clear();
        let _ = self.text.push_str(text);
    }
}

pub struct ProgressBar {
    pub rect: Rectangle,
    pub percent: u32,
    pub rect_colour: Gray8,
    pub fill_colour: Gray8,
}

impl ProgressBar {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        self.rect
            .into_styled(PrimitiveStyle::with_fill(self.rect_colour))
            .draw(display)
            .unwrap();

        if self.percent > 0 {
            let filled = (self.rect.size.width * self.percent) / 100;
            let filled_rect =
                Rectangle::new(self.rect.top_left, Size::new(filled, self.rect.size.height));
            filled_rect
                .into_styled(PrimitiveStyle::with_fill(self.fill_colour))
                .draw(display)
                .unwrap();
        }
    }
}

impl ProgressBar {
    pub fn new(rect: Rectangle, percent: u32, rect_colour: Gray8, fill_colour: Gray8) -> Self {
        Self {
            rect,
            percent,
            rect_colour,
            fill_colour,
        }
    }
}

fn draw_circle(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    center: Point,
    radius: u32,
    style: PrimitiveStyle<Gray8>,
) {
    Circle::with_center(center, radius * 2)
        .into_styled(style)
        .draw(display)
        .unwrap();
}

fn draw_pause_symbol(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    center: Point,
    color: Gray8,
) {
    let style = PrimitiveStyle::with_fill(color);
    Rectangle::new(Point::new(center.x - 5, center.y - 5), Size::new(3, 10))
        .into_styled(style)
        .draw(display)
        .unwrap();
    Rectangle::new(Point::new(center.x + 2, center.y - 5), Size::new(3, 10))
        .into_styled(style)
        .draw(display)
        .unwrap();
}

fn draw_play_symbol(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    center: Point,
    color: Gray8,
) {
    Triangle::new(
        Point::new(center.x - 5, center.y - 5),
        Point::new(center.x - 5, center.y + 5),
        Point::new(center.x + 5, center.y),
    )
    .into_styled(PrimitiveStyle::with_fill(color))
    .draw(display)
    .unwrap();
}

pub struct PlayButton {
    pub center: Point,
    pub radius: u32,
    pub color: Gray8,
    pub playback_state: PlaybackState,
}

impl PlayButton {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        let circle_style = PrimitiveStyleBuilder::new()
            .stroke_color(self.color)
            .stroke_width(2)
            .build();
        draw_circle(display, self.center, self.radius, circle_style);

        match self.playback_state {
            PlaybackState::Playing => draw_pause_symbol(display, self.center, self.color),
            PlaybackState::Paused => draw_play_symbol(display, self.center, self.color),
        }
    }

    pub fn new(center: Point, radius: u32, color: Gray8, playback_state: PlaybackState) -> Self {
        Self {
            center,
            radius,
            color,
            playback_state,
        }
    }
}

pub struct HorizontalLine {
    pub start: Point,
    pub end: Point,
    pub style: PrimitiveStyle<Gray8>,
}

impl HorizontalLine {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        Line::new(self.start, self.end)
            .into_styled(self.style)
            .draw(display)
            .unwrap();
    }
}

impl HorizontalLine {
    pub fn new(start: Point, end: Point, style: PrimitiveStyle<Gray8>) -> Self {
        Self { start, end, style }
    }
}

pub struct List<'a> {
    pub items: &'a [&'a str],
    pub selected: usize,
    pub start: Point,
    pub item_height: i32,
    pub selected_color: Gray8,
    pub unselected_color: Gray8,
    pub selected_prefix: &'a str,
    pub unselected_prefix: &'a str,
}

impl<'a> List<'a> {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        for (i, item) in self.items.iter().enumerate() {
            let y = self.start.y + i as i32 * self.item_height;
            let (color, prefix) = if i == self.selected {
                (self.selected_color, self.selected_prefix)
            } else {
                (self.unselected_color, self.unselected_prefix)
            };

            let prefix_label = Label::new(
                prefix,
                Point::new(self.start.x, y),
                MonoTextStyle::new(&FONT_6X10, color),
            );
            prefix_label.draw(display);

            let item_label = Label::new(
                item,
                Point::new(self.start.x + 12, y),
                MonoTextStyle::new(&FONT_6X10, color),
            );

            item_label.draw(display);
        }
    }
}

pub fn fmt_time_ms(ms: u32) -> [u8; 5] {
    let mut buffer = [0u8; 5];
    let secs = ms / 1000;
    let m = secs / 60;
    let s = secs % 60;
    buffer[0] = b'0' + (m / 10) as u8;
    buffer[1] = b'0' + (m % 10) as u8;
    buffer[2] = b':';
    buffer[3] = b'0' + (s / 10) as u8;
    buffer[4] = b'0' + (s % 10) as u8;
    buffer
}
