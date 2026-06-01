use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use app_core::player::Player;
use app_core::ui::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::ui::event::{Action, Button, Event};
use app_core::ui::screen_manager::ScreenManager;

fn main() {
    let mut display: SimulatorDisplay<Gray8> =
        SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    let output_settings = OutputSettingsBuilder::new()
        .scale(PIXEL_SCALE)
        .pixel_spacing(PIXEL_SPACING)
        .build();

    let mut window = Window::new(
        &format!(
            "Audio Player Simulator — {DISPLAY_WIDTH}×{DISPLAY_HEIGHT} ({PIXEL_SCALE}× scale)"
        ),
        &output_settings,
    );

    let mut player = Player::new(354000);
    let mut screen_manager = ScreenManager::default();

    loop {
        player.tick(33);

        screen_manager.sync(&player);
        screen_manager.draw(&mut display);
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,

                SimulatorEvent::KeyDown { keycode, .. } => {
                    let app_event = match keycode {
                        Keycode::Space => Some(Event {
                            button: Button::Play,
                            action: Action::Pressed,
                        }),
                        Keycode::M => Some(Event {
                            button: Button::Menu,
                            action: Action::Pressed,
                        }),
                        _ => None,
                    };
                    if let Some(e) = app_event {
                        screen_manager.handle_event(&e, &mut player);
                    }
                }

                _ => {}
            }
        }

        // ~30 fps
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
