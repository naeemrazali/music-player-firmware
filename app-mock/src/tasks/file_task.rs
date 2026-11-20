use app_core::{event::Event, file_manager::FileManager};

use crate::{
    mocks::{mock_music_catalogue, mock_sd_card, mock_track_list},
    tasks::{EventChannel, Task},
};

#[embassy_executor::task]
pub async fn run(event_channel: &'static EventChannel) {
    let sd_card = mock_sd_card::new();
    let mut file_manager = FileManager::new(sd_card);

    let sorted_list = mock_track_list::new();
    let music_catalogue = mock_music_catalogue::new();
    file_manager
        .write("/title_asc.lst", &sorted_list)
        .await
        .unwrap();
    file_manager
        .write("/music.cat", &music_catalogue)
        .await
        .unwrap();

    let mut task = Task::new(file_manager, event_channel);

    loop {
        if let embassy_sync::pubsub::WaitResult::Message(event @ Event::FileManager(_)) =
            task.subscriber.next_message().await
        {
            task.service_event(&event).await;
        }
    }
}
