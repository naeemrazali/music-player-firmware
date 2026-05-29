use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Gray8,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle},
    text::Text,
};

pub fn clear_background(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    color: Gray8,
) {
    display.clear(color).unwrap();
}

pub fn draw_line(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    line_start: Point,
    line_end: Point,
    style: PrimitiveStyle<Gray8>,
) {
    Line::new(line_start, line_end)
        .into_styled(style)
        .draw(display)
        .unwrap();
}

pub struct Label<'a> {
    pub text:     &'a str,
    pub position: Point,
    pub style:    MonoTextStyle<'a, Gray8>,
}

pub fn draw_text(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    label: &Label<'_>,
) {
    Text::new(label.text, label.position, label.style)
        .draw(display)
        .unwrap();
}

pub struct List<'a> {
    pub items:             &'a [&'a str],
    pub selected:          usize,
    pub start:             Point,
    pub item_height:       i32,
    pub selected_color:    Gray8,
    pub unselected_color:  Gray8,
    pub selected_prefix:   &'a str,
    pub unselected_prefix: &'a str,
}

pub fn draw_list(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    list: &List<'_>,
) {
    for (i, item) in list.items.iter().enumerate() {
        let y = list.start.y + i as i32 * list.item_height;
        let (color, prefix) = if i == list.selected {
            (list.selected_color, list.selected_prefix)
        } else {
            (list.unselected_color, list.unselected_prefix)
        };
        let prefix_label = Label {
            text:     prefix,
            position: Point::new(list.start.x, y),
            style:    MonoTextStyle::new(&FONT_6X10, color),
        };
        draw_text(display, &prefix_label);

        let item_label = Label {
            text:     item,
            position: Point::new(list.start.x + 12, y),
            style:    MonoTextStyle::new(&FONT_6X10, color),
        };
        draw_text(display, &item_label);
    }
}

pub fn draw_progress_bar(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    total: u32,
    elapsed: u32,
    rect: Rectangle,
    fg: Gray8,
    bg: Gray8,
) {
    // Background track
    rect.into_styled(PrimitiveStyle::with_fill(bg))
        .draw(display)
        .unwrap();

    let percent_elapsed = ((elapsed * 100) / total.max(1)).clamp(0, 100);
    let filled = (rect.size.width * percent_elapsed) / 100;

    if filled > 0 {
        let filled_rect = Rectangle::new(rect.top_left, Size::new(filled, rect.size.height));
        filled_rect
            .into_styled(PrimitiveStyle::with_fill(fg))
            .draw(display)
            .unwrap();
    }
}

pub fn draw_play_button(
    display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    center: Point,
    is_playing: bool,
    color: Gray8,
) {
    let r = 12u32;
    Circle::new(Point::new(center.x - r as i32, center.y - r as i32), r * 2)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(color)
                .stroke_width(2)
                .build(),
        )
        .draw(display)
        .unwrap();

    if is_playing {
        Rectangle::new(Point::new(center.x - 5, center.y - 5), Size::new(3, 10))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)
            .unwrap();
        Rectangle::new(Point::new(center.x + 2, center.y - 5), Size::new(3, 10))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(display)
            .unwrap();
    } else {
        Triangle::new(
            Point::new(center.x - 5, center.y - 5),
            Point::new(center.x - 5, center.y + 5),
            Point::new(center.x + 5, center.y),
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)
        .unwrap();
    }
}

pub fn fmt_time_ms(ms: u32, buf: &mut [u8; 6]) -> &str {
    let secs = ms / 1000;
    let m = secs / 60;
    let s = secs % 60;
    buf[0] = b'0' + (m / 10) as u8;
    buf[1] = b'0' + (m % 10) as u8;
    buf[2] = b':';
    buf[3] = b'0' + (s / 10) as u8;
    buf[4] = b'0' + (s % 10) as u8;
    core::str::from_utf8(&buf[..5]).unwrap_or("00:00")
}
