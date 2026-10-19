use crate::event::{Event, EventHandler, EventQueue, Fetch};

pub struct FileManager {
    events: EventQueue,
}

impl EventHandler for FileManager {
    fn event_queue(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    fn handle_event(&mut self, event: &Event) -> EventQueue {
        if let Event::FileManager(Fetch::Playlist {
            start_index,
            end_index,
        }) = event
        {
            self.fetch_playlist(*start_index, *end_index);
        }
        self.push_events()
    }
}

impl FileManager {
    fn fetch_playlist(&self, start_index: usize, end_index: usize) {}
}
