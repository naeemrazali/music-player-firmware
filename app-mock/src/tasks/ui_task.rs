use embassy_futures::select::{Either, select};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Timer};
use embedded_graphics_simulator::{SimulatorEvent, Window, sdl2::Keycode};

use app_core::{
    event::{Button, Event, Screen},
    ui::screen_manager::ScreenManager,
};

use crate::{
    mocks::{
        mock_display::{self, MockSimulatorDisplay},
        mock_window,
    },
    tasks::{EventChannel, Task},
};

#[embassy_executor::task]
pub async fn run(event_channel: &'static EventChannel) {
    let mut task = Task::new(ScreenManager::default(), event_channel);
    let mut display = mock_display::new();
    let mut window = mock_window::new();

    refresh_screen(task.actor(), &mut window, &mut display);

    loop {
        let timeout = Timer::after(Duration::from_millis(33));

        match select(task.subscriber.next_message(), timeout).await {
            Either::First(WaitResult::Message(event @ Event::Playback(_))) => {
                task.service_event(&event).await;
            }
            Either::First(WaitResult::Message(event @ Event::Ui(Screen::Change(_)))) => {
                task.service_event(&event).await;
            }
            Either::First(WaitResult::Message(Event::Ui(Screen::Refresh))) => {
                refresh_screen(task.actor(), &mut window, &mut display);
            }
            Either::First(_) => {}
            Either::Second(_) => {
                for event in window.events() {
                    match event {
                        SimulatorEvent::Quit => std::process::exit(0),
                        SimulatorEvent::KeyDown { keycode, .. } => {
                            if let Some(button_press) = map_key(&keycode) {
                                task.service_event(&button_press).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn refresh_screen(
    screen_manager: &ScreenManager,
    window: &mut Window,
    display: &mut MockSimulatorDisplay,
) {
    screen_manager.draw(display);
    window.update(display);
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
