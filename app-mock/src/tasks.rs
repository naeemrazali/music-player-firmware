use app_core::{
    event::{Button, Command, Event, Screen},
    player::Player,
    playlist::Playlist,
    track::Track,
    ui::{
        display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING},
        screen_manager::ScreenManager,
    },
};
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::{pixelcolor::Gray8, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};

pub type EventChannel = PubSubChannel<CriticalSectionRawMutex, Event, 64, 2, 2>;

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
pub async fn ui_task(events: &'static EventChannel) {
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

    let mut subscriber = events.subscriber().unwrap();
    let publisher = events.publisher().unwrap();
    let mut screen_manager = ScreenManager::default();
    screen_manager.draw(&mut display);
    window.update(&display);

    loop {
        let timeout = Timer::after(Duration::from_millis(33));

        match select(subscriber.next_message(), timeout).await {
            Either::First(WaitResult::Message(Event::Playback(state))) => {
                for event in screen_manager.handle_event(&Event::Playback(state)) {
                    publisher.publish(event).await;
                }
            }
            Either::First(WaitResult::Message(Event::Ui(Screen::Change(screen)))) => {
                for event in screen_manager.handle_event(&Event::Ui(Screen::Change(screen))) {
                    publisher.publish(event).await;
                }
            }
            Either::First(WaitResult::Message(Event::Ui(Screen::Refresh))) => {
                screen_manager.draw(&mut display);
                window.update(&display);
            }
            Either::First(_) => {}
            Either::Second(_) => {
                for event in window.events() {
                    match event {
                        SimulatorEvent::Quit => std::process::exit(0),
                        SimulatorEvent::KeyDown { keycode, .. } => {
                            if let Some(button_press) = map_key(&keycode) {
                                for event in screen_manager.handle_event(&button_press) {
                                    publisher.publish(event).await;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn player_task(events: &'static EventChannel) {
    let mut player = create_mock_player();
    let mut subscriber = events.subscriber().unwrap();
    let publisher = events.publisher().unwrap();
    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        match select(subscriber.next_message(), ticker.next()).await {
            Either::First(WaitResult::Message(Event::Player(cmd))) => {
                for event in player.handle_event(&Event::Player(cmd)) {
                    publisher.publish(event).await;
                }
            }
            Either::First(_) => {}

            // 100ms elapsed → feed ourselves a Tick command, same door
            Either::Second(_) => {
                for event in player.handle_event(&Event::Player(Command::Tick(100))) {
                    publisher.publish(event).await;
                }
            }
        }
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
