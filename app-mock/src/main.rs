use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use app_core::event::{Button, Event};
use app_core::player::Player;
use app_core::playlist::Playlist;
use app_core::track::Track;
use app_core::ui::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
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
    let _ = playlist.add(Track {
        title: "Gymnopédie No.1",
        artist: "Erik Satie",
        duration_ms: 210000,
        file_path: "/music/flac/gymnopedie_no1.flac",
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
                        Keycode::N => Some(Event::ButtonPress(Button::Next)),
                        Keycode::P => Some(Event::ButtonPress(Button::Prev)),
                        Keycode::Num1 => Some(Event::ButtonPress(Button::Seek(10))),
                        Keycode::Num2 => Some(Event::ButtonPress(Button::Seek(20))),
                        Keycode::Num3 => Some(Event::ButtonPress(Button::Seek(30))),
                        Keycode::Num4 => Some(Event::ButtonPress(Button::Seek(40))),
                        Keycode::Num5 => Some(Event::ButtonPress(Button::Seek(50))),
                        Keycode::Num6 => Some(Event::ButtonPress(Button::Seek(60))),
                        Keycode::Num7 => Some(Event::ButtonPress(Button::Seek(70))),
                        Keycode::Num8 => Some(Event::ButtonPress(Button::Seek(80))),
                        Keycode::Num9 => Some(Event::ButtonPress(Button::Seek(90))),
                        Keycode::Num0 => Some(Event::ButtonPress(Button::Seek(0))),
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
