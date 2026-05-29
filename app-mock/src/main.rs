use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::screens::{AppScreen, MainScreen, SettingsScreen};
use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

fn main() {
    let mut display: SimulatorDisplay<Gray8> =
        SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    let output_settings = OutputSettingsBuilder::new()
        .scale(PIXEL_SCALE)
        .pixel_spacing(PIXEL_SPACING)
        .build();

    let mut window = Window::new(
        &format!(
            "Audio Player Simulator  —  {DISPLAY_WIDTH}×{DISPLAY_HEIGHT}  ({PIXEL_SCALE}× scale)"
        ),
        &output_settings,
    );

    // Mutable state
    let mut is_playing = true;
    let mut elapsed_ms = 0u32;
    let mut on_main_screen = true;

    // Settings screen data is owned independently so we can toggle back and forth
    let settings_items: &[&str] = &["Volume", "Shuffle", "Repeat", "Equalizer"];

    loop {
        // ── Update phase ───────────────────────────────────────────────
        if is_playing {
            elapsed_ms += 33;
            if elapsed_ms > 35400 {
                elapsed_ms = 35400;
            }
        }

        // Build the current screen from mutable state
        let mut current = if on_main_screen {
            AppScreen::Main(MainScreen {
                background: Gray8::new(0x9F),
                title_label: app_core::ui::Label {
                    text: "Clair de Lune",
                    position: Point::new(10, 20),
                    style: MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
                },
                artist_label: app_core::ui::Label {
                    text: "Claude Debussy",
                    position: Point::new(10, 36),
                    style: MonoTextStyle::new(&FONT_6X10, Gray8::new(100)),
                },
                elapsed_ms,
                total_ms: 35400,
                is_playing,
                volume: 70,
                elapsed_time_buf: [0u8; 6],
                total_time_buf: [0u8; 6],
                default_text_style: MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
            })
        } else {
            AppScreen::Settings(SettingsScreen {
                background: Gray8::new(0x9F),
                items: settings_items,
                selected: 0,
                heading: app_core::ui::Label {
                    text: "Settings",
                    position: Point::new(10, 20),
                    style: MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
                },
            })
        };

        // ── Draw phase ───────────────────────────────────────────────────
        current.draw(&mut display);
        window.update(&display);

        // ── Input phase ──────────────────────────────────────────────────
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,

                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Space => is_playing = !is_playing,
                    Keycode::M => on_main_screen = !on_main_screen,
                    Keycode::Escape => return,
                    _ => {}
                },

                _ => {}
            }
        }

        // ~30 fps
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
