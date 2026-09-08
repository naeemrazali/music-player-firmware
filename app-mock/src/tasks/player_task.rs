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
    const TICK_DURATION: u64 = 100;

    let mut task = Task::new(mock_player::new(), event_channel);
    let mut ticker = Ticker::every(Duration::from_millis(TICK_DURATION));

    loop {
        match select(task.subscriber.next_message(), ticker.next()).await {
            Either::First(WaitResult::Message(event @ Event::Player(_))) => {
                task.service_event(&event).await;
            }
            Either::First(_) => {}
            Either::Second(_) => {
                task.service_event(&Event::Player(Command::Tick(TICK_DURATION as u32)))
                    .await;
            }
        }
    }
}
