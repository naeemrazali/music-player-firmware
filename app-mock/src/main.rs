use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::player::Player;
use app_core::screen_manager::ScreenManager;
use app_core::ui;

use core::str::from_utf8;

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

    let mut player = Player::new(354000);
    let mut screen_manager = ScreenManager::default();

    loop {
        player.tick(33);

        screen_manager.main.progress.percent = player.progress_percent();
        screen_manager
            .main
            .total_time
            .set_text(from_utf8(&ui::fmt_time_ms(player.total_ms())).unwrap());
        screen_manager
            .main
            .elapsed_time
            .set_text(from_utf8(&ui::fmt_time_ms(player.elapsed_ms())).unwrap());

        screen_manager.draw(&mut display);
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,

                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Space => {
                        screen_manager.main.play_button.playback_state = player.toggle_playback()
                    }
                    Keycode::M => screen_manager.toggle_screens(),
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
