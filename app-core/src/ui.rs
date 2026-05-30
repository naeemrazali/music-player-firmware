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

pub struct Label<'a> {
    pub text:     &'a str,
    pub position: Point,
    pub style:    MonoTextStyle<'a, Gray8>,
}

impl<'a> Label<'a> {
    pub fn draw(&self, display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>) {
        Text::new(self.text, self.position, self.style)
            .draw(display)
            .unwrap();
    }
}

pub struct ProgressBar {
    pub rect:       Rectangle,
    pub fg:         Gray8,
    pub bg:         Gray8,
    pub total_ms:   u32,
    pub elapsed_ms: u32,
}

impl ProgressBar {
    pub fn draw(&self, display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>) {
        self.rect
            .into_styled(PrimitiveStyle::with_fill(self.bg))
            .draw(display)
            .unwrap();

        let percent = ((self.elapsed_ms * 100) / self.total_ms.max(1)).clamp(0, 100);
        let filled  = (self.rect.size.width * percent) / 100;

        if filled > 0 {
            let filled_rect = Rectangle::new(
                self.rect.top_left,
                Size::new(filled, self.rect.size.height),
            );
            filled_rect
                .into_styled(PrimitiveStyle::with_fill(self.fg))
                .draw(display)
                .unwrap();
        }
    }
}

pub struct PlayButton {
    pub center:     Point,
    pub color:      Gray8,
    pub is_playing: bool,
}

impl PlayButton {
    pub fn draw(&self, display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>) {
        let r = 12u32;
        Circle::new(
            Point::new(self.center.x - r as i32, self.center.y - r as i32),
            r * 2,
        )
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(self.color)
                .stroke_width(2)
                .build(),
        )
        .draw(display)
        .unwrap();

        if self.is_playing {
            Rectangle::new(
                Point::new(self.center.x - 5, self.center.y - 5),
                Size::new(3, 10),
            )
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(display)
            .unwrap();
            Rectangle::new(
                Point::new(self.center.x + 2, self.center.y - 5),
                Size::new(3, 10),
            )
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(display)
            .unwrap();
        } else {
            Triangle::new(
                Point::new(self.center.x - 5, self.center.y - 5),
                Point::new(self.center.x - 5, self.center.y + 5),
                Point::new(self.center.x + 5, self.center.y),
            )
            .into_styled(PrimitiveStyle::with_fill(self.color))
            .draw(display)
            .unwrap();
        }
    }
}

pub struct HorizontalLine {
    pub start: Point,
    pub end:   Point,
    pub style: PrimitiveStyle<Gray8>,
}

impl HorizontalLine {
    pub fn draw(&self, display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>) {
        Line::new(self.start, self.end)
            .into_styled(self.style)
            .draw(display)
            .unwrap();
    }
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

impl<'a> List<'a> {
    pub fn draw(&self, display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>) {
        for (i, item) in self.items.iter().enumerate() {
            let y = self.start.y + i as i32 * self.item_height;
            let (color, prefix) = if i == self.selected {
                (self.selected_color, self.selected_prefix)
            } else {
                (self.unselected_color, self.unselected_prefix)
            };

            let prefix_label = Label {
                text:     prefix,
                position: Point::new(self.start.x, y),
                style:    MonoTextStyle::new(&FONT_6X10, color),
            };
            prefix_label.draw(display);

            let item_label = Label {
                text:     item,
                position: Point::new(self.start.x + 12, y),
                style:    MonoTextStyle::new(&FONT_6X10, color),
            };
            item_label.draw(display);
        }
    }
}

pub enum Widget<'a> {
    Label(Label<'a>),
    ProgressBar(ProgressBar),
    PlayButton(PlayButton),
    HorizontalLine(HorizontalLine),
    List(List<'a>),
}

impl<'a> Widget<'a> {
    pub fn draw(&self, display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>) {
        match self {
            Widget::Label(w)        => w.draw(display),
            Widget::ProgressBar(w)  => w.draw(display),
            Widget::PlayButton(w)   => w.draw(display),
            Widget::HorizontalLine(w) => w.draw(display),
            Widget::List(w)         => w.draw(display),
        }
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
