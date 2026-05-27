use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Gray8,
    prelude::*,
    text::Text,
};

pub fn clear_background(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    color: Gray8,
) {
    display.clear(color).unwrap();
}

pub fn draw_text(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    text: &str,
    position: Point,
    color: Gray8,
) {
    let style = MonoTextStyle::new(&FONT_6X10, color);
    Text::new(text, position, style).draw(display).unwrap();
}
