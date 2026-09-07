use embassy_futures::select::{Either, select};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Ticker};

use app_core::event::{Command, Event};

use crate::{
    mocks::mock_player,
    tasks::{EventChannel, Task},
};

#[embassy_executor::task]
pub async fn run(event_channel: &'static EventChannel) {
    let mut task = Task::new(event_channel);
    let mut player = mock_player::new();
    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        match select(task.subscriber.next_message(), ticker.next()).await {
            Either::First(WaitResult::Message(Event::Player(cmd))) => {
                task.service_event(&mut player, &Event::Player(cmd)).await;
            }
            Either::First(_) => {}
            Either::Second(_) => {
                task.service_event(&mut player, &Event::Player(Command::Tick(100)))
                    .await;
            }
        }
    }
}
