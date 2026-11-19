use app_core::{event::Event, file_manager::FileManager};

use crate::{
    mocks::mock_sd_card::MockBlockDevice,
    tasks::{EventChannel, Task},
};

#[embassy_executor::task]
pub async fn run(event_channel: &'static EventChannel) {
    // need the device D, which is the mock SD card
    let sd_card = MockBlockDevice::new();
    let mut task = Task::new(FileManager::new(sd_card), event_channel);

    loop {
        if let embassy_sync::pubsub::WaitResult::Message(event @ Event::FileManager(_)) =
            task.subscriber.next_message().await
        {
            task.service_event(&event).await;
        }
    }
}
