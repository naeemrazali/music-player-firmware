use alloc::boxed::Box;

use crate::{
    event::{Command, Event, EventHandler, EventQueue, Fetch},
    playlist::{Playlist, SortOrder},
};

pub struct FileManager {
    events: EventQueue,
}

impl EventHandler for FileManager {
    fn event_queue(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    fn handle_event(&mut self, event: &Event) -> EventQueue {
        if let Event::FileManager(Fetch::Playlist {
            sort_order,
            start_index,
            end_index,
        }) = event
        {
            self.fetch_playlist(*sort_order, *start_index, *end_index);
        }
        self.push_events()
    }
}

impl FileManager {
    fn fetch_playlist(&mut self, sort_order: SortOrder, start_index: usize, end_index: usize) {
        // create an empty playlist
        // for start to end index read sorted list
        // add new item from sorted list to playlist
        // add event update playlist with new playlist

        let playlist = Playlist::new();
        self.add_event(Event::Player(Command::UpdatePlaylist(Box::new(playlist))));
    }
}
