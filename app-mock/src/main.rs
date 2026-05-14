mod display_config;
mod ui;

use display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use ui::{draw_player_screen, PlayerState};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window};

fn main() {
    // ── Create the simulated display ─────────────────────────────────────────
    // SimulatorDisplay<Rgb565> is a DrawTarget — the same trait implemented by
    // real display drivers (mipidsi, st7789, ili9341, etc.).
    // Swapping this out for a real driver in app-firmware requires no changes
    // to any code in ui.rs.
    let mut display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    // ── Configure the desktop window ─────────────────────────────────────────
    // PIXEL_SCALE zooms every simulated pixel up to a comfortable desktop size.
    // The window title shows the real resolution so you always know what you
    // are targeting.
    let output_settings = OutputSettingsBuilder::new()
        .scale(PIXEL_SCALE)
        .pixel_spacing(PIXEL_SPACING)
        .build();

    let mut window = Window::new(
        &format!(
            "Audio Player Simulator  —  {}×{}  ({}× scale)",
            DISPLAY_WIDTH, DISPLAY_HEIGHT, PIXEL_SCALE
        ),
        &output_settings,
    );

    // ── Demo player state ─────────────────────────────────────────────────────
    // Replace with real state from app-core once the player logic is wired in.
    let mut elapsed = 42u32;
    let mut playing = true;

    // ── Event loop ────────────────────────────────────────────────────────────
    // The loop mirrors the structure of the real firmware display task:
    //   1. Build current state
    //   2. Draw the frame
    //   3. Handle input events
    //
    // On hardware step 3 is replaced by reading GPIO button states.
    'running: loop {
        let state = PlayerState {
            track_title:  "Clair de Lune",
            artist:       "Claude Debussy",
            elapsed_secs: elapsed,
            total_secs:   354,
            volume:       70,
            is_playing:   playing,
        };

        // Draw the full UI into the simulated framebuffer
        draw_player_screen(
            &mut display,
            &state,
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            "FLAC",
        );

        // Push the framebuffer to the SDL2 window
        window.update(&display);

        // Handle SDL2 events
        for event in window.events() {
            match event {
                // Close button or Escape — exit
                SimulatorEvent::Quit => break 'running,

                // Keyboard shortcuts to test UI states during development:
                //   Space  → toggle play/pause
                //   Right  → advance 5 seconds
                //   Left   → rewind 5 seconds
                SimulatorEvent::KeyDown { keycode, .. } => {
                    use embedded_graphics_simulator::sdl2::Keycode;
                    match keycode {
                        Keycode::Space => {
                            playing = !playing;
                        }
                        Keycode::Right => {
                            elapsed = (elapsed + 5).min(354);
                        }
                        Keycode::Left => {
                            elapsed = elapsed.saturating_sub(5);
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        // Advance time when playing (simulates the real passage of audio)
        if playing {
            elapsed = (elapsed + 1).min(354);
        }

        // ~30 fps
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
