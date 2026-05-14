use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};

pub const COL_BACKGROUND: Rgb565 = Rgb565::BLACK;
pub const COL_SURFACE:    Rgb565 = Rgb565::new(4, 8, 12);
pub const COL_ACCENT:     Rgb565 = Rgb565::new(0, 25, 31);
pub const COL_PRIMARY:    Rgb565 = Rgb565::WHITE;
pub const COL_SECONDARY:  Rgb565 = Rgb565::new(20, 40, 20);
pub const COL_PROGRESS:   Rgb565 = Rgb565::new(0, 52, 10);
pub const COL_TRACK_BG:   Rgb565 = Rgb565::new(6, 12, 6);

pub struct PlayerState<'a> {
    pub track_title:  &'a str,
    pub artist:       &'a str,
    pub elapsed_secs: u32,
    pub total_secs:   u32,
    pub volume:       u8,
    pub is_playing:   bool,
}

impl<'a> PlayerState<'a> {
    pub fn progress(&self) -> f32 {
        if self.total_secs == 0 {
            return 0.0;
        }
        (self.elapsed_secs as f32 / self.total_secs as f32).clamp(0.0, 1.0)
    }
}

fn fmt_time(secs: u32, buf: &mut [u8; 6]) -> &str {
    let m = secs / 60;
    let s = secs % 60;
    buf[0] = b'0' + (m / 10) as u8;
    buf[1] = b'0' + (m % 10) as u8;
    buf[2] = b':';
    buf[3] = b'0' + (s / 10) as u8;
    buf[4] = b'0' + (s % 10) as u8;
    buf[5] = 0;
    core::str::from_utf8(&buf[..5]).unwrap_or("00:00")
}

pub fn draw_background(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
) {
    display.clear(COL_BACKGROUND).unwrap();
}

pub fn draw_track_info(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width: i32,
) {
    let artist_style = MonoTextStyle::new(&FONT_6X10, COL_SECONDARY);
    let title_style  = MonoTextStyle::new(&FONT_10X20, COL_PRIMARY);

    Text::with_alignment(
        state.artist,
        Point::new(display_width / 2, 22),
        artist_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();

    Text::with_alignment(
        state.track_title,
        Point::new(display_width / 2, 50),
        title_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}

pub fn draw_progress(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width: i32,
    display_height: i32,
) {
    let bar_margin = 20i32;
    let bar_y      = display_height - 60;
    let bar_height = 6i32;
    let bar_width  = display_width - bar_margin * 2;

    Rectangle::new(
        Point::new(bar_margin, bar_y),
        Size::new(bar_width as u32, bar_height as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(COL_TRACK_BG))
    .draw(display)
    .unwrap();

    let filled = (bar_width as f32 * state.progress()) as u32;
    if filled > 0 {
        Rectangle::new(
            Point::new(bar_margin, bar_y),
            Size::new(filled, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(COL_PROGRESS))
        .draw(display)
        .unwrap();
    }

    let dot_x = bar_margin + filled as i32;
    Circle::new(Point::new(dot_x - 4, bar_y - 3), 10)
        .into_styled(PrimitiveStyle::with_fill(COL_PRIMARY))
        .draw(display)
        .unwrap();

    let time_style = MonoTextStyle::new(&FONT_6X10, COL_SECONDARY);
    let mut elapsed_buf = [0u8; 6];
    let mut total_buf   = [0u8; 6];
    let elapsed_str = fmt_time(state.elapsed_secs, &mut elapsed_buf);
    let total_str   = fmt_time(state.total_secs,   &mut total_buf);

    Text::new(elapsed_str, Point::new(bar_margin, bar_y + 18), time_style)
        .draw(display)
        .unwrap();

    Text::with_alignment(
        total_str,
        Point::new(display_width - bar_margin, bar_y + 18),
        time_style,
        Alignment::Right,
    )
    .draw(display)
    .unwrap();
}

pub fn draw_transport(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width:  i32,
    display_height: i32,
) {
    let cx = display_width  / 2;
    let cy = display_height / 2 + 10;
    let r  = 28u32;

    Circle::new(Point::new(cx - r as i32, cy - r as i32), r * 2)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(COL_ACCENT)
                .stroke_width(2)
                .fill_color(COL_SURFACE)
                .build(),
        )
        .draw(display)
        .unwrap();

    if state.is_playing {
        Rectangle::new(Point::new(cx - 12, cy - 12), Size::new(8, 24))
            .into_styled(PrimitiveStyle::with_fill(COL_PRIMARY))
            .draw(display)
            .unwrap();
        Rectangle::new(Point::new(cx + 4, cy - 12), Size::new(8, 24))
            .into_styled(PrimitiveStyle::with_fill(COL_PRIMARY))
            .draw(display)
            .unwrap();
    } else {
        let style = PrimitiveStyle::with_stroke(COL_PRIMARY, 2);
        for i in 0i32..12 {
            Line::new(
                Point::new(cx - 8,      cy - 12 + i),
                Point::new(cx - 8 + i,  cy),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();
            Line::new(
                Point::new(cx - 8,      cy + 12 - i),
                Point::new(cx - 8 + i,  cy),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();
        }
    }
}

pub fn draw_volume(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_height: i32,
) {
    let label_style = MonoTextStyle::new(&FONT_6X10, COL_SECONDARY);
    let vol_y       = display_height - 30;

    Text::new("VOL", Point::new(4, vol_y), label_style)
        .draw(display)
        .unwrap();

    for i in 0u8..10 {
        let filled  = (state.volume / 10) > i;
        let bar_col = if filled { COL_ACCENT } else { COL_TRACK_BG };
        let x       = 32 + i as i32 * 10;
        let h       = 4 + i as i32 * 1;
        Rectangle::new(
            Point::new(x, vol_y - h + 4),
            Size::new(7, h as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(bar_col))
        .draw(display)
        .unwrap();
    }
}

pub fn draw_format_badge(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    format: &str,
    display_width: i32,
) {
    let badge_style = MonoTextStyle::new(&FONT_6X10, COL_ACCENT);
    Text::with_alignment(
        format,
        Point::new(display_width - 6, 12),
        badge_style,
        Alignment::Right,
    )
    .draw(display)
    .unwrap();
}

pub fn draw_player_screen(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width:  u32,
    display_height: u32,
    format: &str,
) {
    let w = display_width  as i32;
    let h = display_height as i32;

    draw_background(display);
    draw_track_info(display, state, w);
    draw_format_badge(display, format, w);
    draw_transport(display, state, w, h);
    draw_progress(display, state, w, h);
    draw_volume(display, state, h);
}
