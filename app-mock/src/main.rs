use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::screens::Screen;
use app_core::ui::draw;

use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};

fn main() {
    let mut display: SimulatorDisplay<Gray8> =
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

    let screen = Screen {
        background: Gray8::new(255),
    };

    draw(&mut display, &screen);

    window.update(&display);

    // Keep the window open until the user closes it
    loop {
        if window
            .events()
            .any(|e| matches!(e, embedded_graphics_simulator::SimulatorEvent::Quit))
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
