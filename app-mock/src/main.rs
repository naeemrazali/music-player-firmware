use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::ui::{draw_player_screen, PlayerState};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

fn main() {
    let mut display: SimulatorDisplay<Rgb565> =
        SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

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

    let state = PlayerState {
        track_title:  "Clair de Lune",
        artist:       "Claude Debussy",
        elapsed_secs: 42,
        total_secs:   354,
        volume:       70,
        is_playing:   true,
    };

    draw_player_screen(
        &mut display,
        &state,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        "FLAC",
    );

    window.update(&display);

    // Keep the window open until the user closes it
    loop {
        if window.events().any(|e| matches!(e, embedded_graphics_simulator::SimulatorEvent::Quit)) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
