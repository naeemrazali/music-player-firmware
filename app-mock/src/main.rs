use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::screens::{AppScreen, MainScreen};
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
            "Audio Player Simulator  —  {DISPLAY_WIDTH}×{DISPLAY_HEIGHT}  ({PIXEL_SCALE}× scale)"
        ),
        &output_settings,
    );

    let current = AppScreen::Main(MainScreen {
        background: Gray8::new(0x9F),
        title: "Clair de Lune",
        artist: "Claude Debussy",
    });

    current.draw(&mut display);

    window.update(&display);

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
