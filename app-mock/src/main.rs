use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::screens::{AppScreen, MainScreen, SettingsScreen};
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

    // test constant
    let total_ms = 35400;

    // Pre-construct both screens
    let mut main_screen = MainScreen::default();
    let mut settings_screen = SettingsScreen::default();

    loop {
        // ── Update phase ───────────────────────────────────────────────
        if is_playing {
            elapsed_ms += 33;
            if elapsed_ms > total_ms {
                elapsed_ms = total_ms;
            }
        }

        // Sync state into main_screen widgets
        main_screen.play_button.is_playing = is_playing;
        main_screen.progress.percent = (elapsed_ms * 100) / total_ms;

        // ── Draw phase ───────────────────────────────────────────────────
        let current = if on_main_screen {
            AppScreen::Main(&mut main_screen)
        } else {
            AppScreen::Settings(&mut settings_screen)
        };
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
