use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use heapless::Vec;

use app_core::event::{Button, Event};
use app_core::player::Player;
use app_core::playlist::Playlist;
use app_core::track::Track;
use app_core::ui::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::ui::screen_manager::ScreenManager;

static EVENTS_CH: Channel<CriticalSectionRawMutex, Event, 8> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(player_task()).unwrap();
    spawner.spawn(ui_task()).unwrap();
}

fn create_mock_player() -> Player {
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
    Player::new(playlist)
}

#[embassy_executor::task]
async fn player_task() {
    let mut player = create_mock_player();
    drain_to(player.event_queue(), &EVENTS_CH).await;

    let mut ticker = Ticker::every(Duration::from_millis(33));

    loop {
        ticker.next().await;

        while let Ok(event) = EVENTS_CH.try_receive() {
            player.handle_event(&event);
        }

        player.tick(33);
        drain_to(player.event_queue(), &EVENTS_CH).await;
    }
}

#[embassy_executor::task]
async fn ui_task() {
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

    let mut screen_manager = ScreenManager::default();

    loop {
        while let Ok(event) = EVENTS_CH.try_receive() {
            screen_manager.handle_event(&event);
        }

        drain_to(screen_manager.event_queue(), &EVENTS_CH).await;

        if screen_manager.needs_refresh {
            screen_manager.draw(&mut display);
            window.update(&display);
            screen_manager.needs_refresh = false;
        }

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => std::process::exit(0),
                SimulatorEvent::KeyDown { keycode, .. } => {
                    if let Some(e) = map_key(&keycode) {
                        screen_manager.handle_event(&e);
                    }
                }
                _ => {}
            }
        }

        Timer::after_millis(10).await;
    }
}

async fn drain_to(event_queue: Vec<Event, 8>, ch: &Channel<CriticalSectionRawMutex, Event, 8>) {
    for event in event_queue {
        ch.send(event).await;
    }
}

fn map_key(keycode: &Keycode) -> Option<Event> {
    match *keycode {
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
    }
}
