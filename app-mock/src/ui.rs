use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, ascii::FONT_7X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
        CornerRadii,
    },
    text::{Alignment, Text},
};

// ── Colour palette ───────────────────────────────────────────────────────────
// Defined once here so the same constants work on real hardware and simulator.

pub const COL_BACKGROUND: Rgb565 = Rgb565::BLACK;
pub const COL_SURFACE:    Rgb565 = Rgb565::new(4, 8, 12);     // dark navy panel
pub const COL_ACCENT:     Rgb565 = Rgb565::new(0, 25, 31);    // teal highlight
pub const COL_PRIMARY:    Rgb565 = Rgb565::WHITE;
pub const COL_SECONDARY:  Rgb565 = Rgb565::new(20, 40, 20);   // muted grey-green
pub const COL_PROGRESS:   Rgb565 = Rgb565::new(0, 52, 10);    // progress bar fill
pub const COL_TRACK_BG:   Rgb565 = Rgb565::new(6, 12, 6);     // progress bar track

// ── State passed into draw calls ─────────────────────────────────────────────

/// Represents the player state that the UI needs to render.
/// In the real firmware this will be owned by app-core's Player struct
/// and passed to the display task. It is defined here so the simulator
/// can drive it with test values.
pub struct PlayerState<'a> {
    pub track_title:  &'a str,
    pub artist:       &'a str,
    pub elapsed_secs: u32,
    pub total_secs:   u32,
    pub volume:       u8,    // 0–100
    pub is_playing:   bool,
}

impl<'a> PlayerState<'a> {
    /// Returns elapsed / total as a value in 0.0–1.0.
    pub fn progress(&self) -> f32 {
        if self.total_secs == 0 {
            return 0.0;
        }
        (self.elapsed_secs as f32 / self.total_secs as f32).clamp(0.0, 1.0)
    }
}

// ── Drawing helpers ──────────────────────────────────────────────────────────

/// Format seconds as MM:SS into a fixed-size stack buffer.
/// Returns a &str slice into `buf`.
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

// ── Screen drawing functions ─────────────────────────────────────────────────
// Each function takes a `&mut impl DrawTarget<Color = Rgb565>` so the exact
// same code compiles against SimulatorDisplay or a real mipidsi/ST7789 driver.

/// Fill the entire display with the background colour.
pub fn draw_background(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
) {
    display.clear(COL_BACKGROUND).unwrap();
}

/// Draw the top panel: artist name above, track title in larger font below.
pub fn draw_track_info(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width: i32,
) {
    let artist_style = MonoTextStyle::new(&FONT_6X10, COL_SECONDARY);
    let title_style  = MonoTextStyle::new(&FONT_10X20, COL_PRIMARY);

    // Artist — centred, near top
    Text::with_alignment(
        state.artist,
        Point::new(display_width / 2, 22),
        artist_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();

    // Track title — centred below artist
    Text::with_alignment(
        state.track_title,
        Point::new(display_width / 2, 50),
        title_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}

/// Draw the progress bar and elapsed / remaining time labels.
pub fn draw_progress(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width: i32,
    display_height: i32,
) {
    let bar_margin  = 20i32;
    let bar_y       = display_height - 60;
    let bar_height  = 6i32;
    let bar_width   = display_width - bar_margin * 2;

    // Background track
    Rectangle::new(
        Point::new(bar_margin, bar_y),
        Size::new(bar_width as u32, bar_height as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(COL_TRACK_BG))
    .draw(display)
    .unwrap();

    // Filled portion
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

    // Playhead dot at current position
    let dot_x = bar_margin + filled as i32;
    Circle::new(Point::new(dot_x - 4, bar_y - 3), 10)
        .into_styled(PrimitiveStyle::with_fill(COL_PRIMARY))
        .draw(display)
        .unwrap();

    // Time labels
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

/// Draw the play/pause button indicator in the centre of the screen.
pub fn draw_transport(
    display: &mut impl DrawTarget<Color = Rgb565, Error = impl core::fmt::Debug>,
    state: &PlayerState,
    display_width:  i32,
    display_height: i32,
) {
    let cx = display_width  / 2;
    let cy = display_height / 2 + 10;
    let r  = 28u32;

    // Outer circle
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
        // Pause icon — two vertical bars
        Rectangle::new(Point::new(cx - 12, cy - 12), Size::new(8, 24))
            .into_styled(PrimitiveStyle::with_fill(COL_PRIMARY))
            .draw(display)
            .unwrap();
        Rectangle::new(Point::new(cx + 4, cy - 12), Size::new(8, 24))
            .into_styled(PrimitiveStyle::with_fill(COL_PRIMARY))
            .draw(display)
            .unwrap();
    } else {
        // Play icon — right-pointing triangle (drawn as lines)
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

    // Previous / next skip arrows either side of the play button
    let skip_style = PrimitiveStyle::with_stroke(COL_SECONDARY, 2);

    // << Previous
    Line::new(Point::new(cx - 60, cy), Point::new(cx - 48, cy - 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
    Line::new(Point::new(cx - 60, cy), Point::new(cx - 48, cy + 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
    Line::new(Point::new(cx - 52, cy), Point::new(cx - 40, cy - 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
    Line::new(Point::new(cx - 52, cy), Point::new(cx - 40, cy + 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();

    // >> Next
    Line::new(Point::new(cx + 60, cy), Point::new(cx + 48, cy - 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
    Line::new(Point::new(cx + 60, cy), Point::new(cx + 48, cy + 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
    Line::new(Point::new(cx + 52, cy), Point::new(cx + 40, cy - 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
    Line::new(Point::new(cx + 52, cy), Point::new(cx + 40, cy + 10))
        .into_styled(skip_style)
        .draw(display)
        .unwrap();
}

/// Draw the volume bar in the bottom-left corner.
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

    // 10 small volume notch bars
    for i in 0u8..10 {
        let filled  = (state.volume / 10) > i;
        let bar_col = if filled { COL_ACCENT } else { COL_TRACK_BG };
        let x       = 32 + i as i32 * 10;
        let h       = 4 + i as i32 * 1; // slight taper upward
        Rectangle::new(
            Point::new(x, vol_y - h + 4),
            Size::new(7, h as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(bar_col))
        .draw(display)
        .unwrap();
    }
}

/// Draw the audio format badge (e.g. "FLAC", "MP3") in the top-right corner.
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

/// Render the complete player UI in one call.
/// This is the function your firmware display task will call on every frame.
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
