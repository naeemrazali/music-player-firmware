use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use app_core::player::Player;
use app_core::playlist::Playlist;
use app_core::track::Track;
use app_core::ui::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::ui::event::{Button, Event};
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

    let mut playlist = Playlist::new();
    let _ = playlist.add(Track {
        title: "Clair de Lune",
        artist: "Claude Debussy",
        duration_ms: 354000,
        file_path: "/music/flac/clair_de_lune.flac",
    });

    let mut player = Player::new(playlist);
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
                        Keycode::Space => Some(Event::ButtonPress(Button::Play)),
                        Keycode::M => Some(Event::ButtonPress(Button::Menu)),
                        Keycode::Num1 => Some(Event::Seek(player.total_ms() / 10)),
                        Keycode::Num2 => Some(Event::Seek(player.total_ms() * 2 / 10)),
                        Keycode::Num3 => Some(Event::Seek(player.total_ms() * 3 / 10)),
                        Keycode::Num4 => Some(Event::Seek(player.total_ms() * 4 / 10)),
                        Keycode::Num5 => Some(Event::Seek(player.total_ms() * 5 / 10)),
                        Keycode::Num6 => Some(Event::Seek(player.total_ms() * 6 / 10)),
                        Keycode::Num7 => Some(Event::Seek(player.total_ms() * 7 / 10)),
                        Keycode::Num8 => Some(Event::Seek(player.total_ms() * 8 / 10)),
                        Keycode::Num9 => Some(Event::Seek(player.total_ms() * 9 / 10)),
                        Keycode::Num0 => Some(Event::Seek(0)),
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
