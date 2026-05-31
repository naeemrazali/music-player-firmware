use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::player::Player;
use app_core::screens::{AppScreen, MainScreen, SettingsScreen};
use app_core::ui;
use core::str::from_utf8;
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

    let mut on_main_screen = true;

    let mut player = Player::new(354000);

    // Pre-construct both screens
    let mut main_screen = MainScreen::default();
    let mut settings_screen = SettingsScreen::default();

    loop {
        // ── Update phase ───────────────────────────────────────────────
        player.tick(33);

        // Sync state into main_screen widgets
        main_screen.play_button.is_playing = player.is_playing();
        main_screen.progress.percent = player.progress_percent();
        main_screen
            .total_time
            .set_text(from_utf8(&ui::fmt_time_ms(player.total_ms())).unwrap());
        main_screen
            .elapsed_time
            .set_text(from_utf8(&ui::fmt_time_ms(player.elapsed_ms())).unwrap());

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
                    Keycode::Space => player.toggle_playback(),
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
