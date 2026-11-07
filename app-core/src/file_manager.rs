pub mod filesystem_io;
pub mod track_list;

use alloc::boxed::Box;
use exfat_slim::asynchronous::{BlockDevice, file_system::FileSystem};

use crate::{
    event::{Command, Event, EventHandler, EventQueue, Fetch},
    file_manager::{
        filesystem_io::{Fs, SD_BLOCK_SIZE},
        track_list::TrackList,
    },
    playlist::{Playlist, SortOrder},
};

pub struct FileManager<D: BlockDevice<SD_BLOCK_SIZE>> {
    events: EventQueue,
    fs: Fs<D>,
    track_list: TrackList,
}

impl<D: BlockDevice<SD_BLOCK_SIZE>> EventHandler for FileManager<D> {
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

impl<D: BlockDevice<SD_BLOCK_SIZE>> FileManager<D> {
    pub fn new(device: D) -> Self {
        Self {
            events: EventQueue::new(),
            fs: FileSystem::new(device),
            track_list: TrackList::default(),
        }
    }

    fn fetch_playlist(&mut self, sort_order: SortOrder, start_index: usize, end_index: usize) {
        // create an empty playlist
        //
        // open sorted list
        // check if sorted list exists
        // if does not exist return early
        //
        // for start to end index read sorted list
        // add new item from sorted list to playlist
        // add event update playlist with new playlist

        // TODO(m4): sort-order semantics live here — Descending inversion
        // (num_items - 1 - logical), end-of-library clamping, and the open-once
        // header read; TrackList::fetch_track_index takes physical indices only
        let playlist = Playlist::new();
        self.track_list = TrackList::new(sort_order);

        self.add_event(Event::Player(Command::UpdatePlaylist(Box::new(playlist))));
    }
}
