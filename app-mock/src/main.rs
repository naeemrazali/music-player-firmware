mod mocks;
mod tasks;

use embassy_executor::Spawner;
use embassy_sync::pubsub::PubSubChannel;

use crate::tasks::{EventChannel, player_task, ui_task};

static EVENTS: EventChannel = PubSubChannel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(ui_task(&EVENTS)).unwrap();
    spawner.spawn(player_task(&EVENTS)).unwrap();
}
