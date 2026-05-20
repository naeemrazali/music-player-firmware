use crate::screens::Screen;
use embedded_graphics::{pixelcolor::Gray8, prelude::*};

pub fn draw(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    screen: &Screen,
) {
    display.clear(screen.background).unwrap();
}
