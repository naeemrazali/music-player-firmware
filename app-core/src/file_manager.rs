pub mod filesystem_io;
pub mod header;
pub mod helpers;
pub mod music_catalogue;
pub mod track_list;

use alloc::boxed::Box;
use exfat_slim::asynchronous::{BlockDevice, file_system::FileSystem};

use crate::{
    event::{Command, Event, EventHandler, EventQueue, Fetch},
    file_manager::{
        filesystem_io::{Fs, FsError, SD_BLOCK_SIZE},
        music_catalogue::MusicCatalogue,
        track_list::TrackList,
    },
    playlist::{Playlist, SortOrder},
};

pub struct FileManager<D: BlockDevice<SD_BLOCK_SIZE>> {
    events: EventQueue,
    fs: Fs<D>,
    track_list: TrackList,
    music_catalogue: MusicCatalogue,
}

impl<D: BlockDevice<SD_BLOCK_SIZE>> EventHandler for FileManager<D> {
    fn event_queue(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    async fn handle_event(&mut self, event: &Event) -> EventQueue {
        if let Event::FileManager(Fetch::Playlist {
            sort_order,
            start_index,
            end_index,
        }) = event
        {
            match self
                .fetch_playlist(*sort_order, *start_index, *end_index)
                .await
            {
                // TODO(functionality): returns empty playlist on any error
                // might want to return whatever we can parse and leave out the rest
                Ok(playlist) => self.add_event(Event::Player(Command::UpdatePlaylist(Some(
                    Box::new(playlist),
                )))),
                Err(_) => self.add_event(Event::Player(Command::UpdatePlaylist(None))),
            }
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
            music_catalogue: MusicCatalogue::default(),
        }
    }

    async fn fetch_playlist(
        &mut self,
        sort_order: SortOrder,
        start_index: usize,
        end_index: usize,
    ) -> Result<Playlist, FsError> {
        // TODO(m4): sort-order semantics live here — Descending inversion
        // (num_items - 1 - logical), end-of-library clamping, and the open-once
        // header read; TrackList::fetch_track_index takes physical indices only
        let mut playlist = Playlist::new();
        self.track_list = TrackList::new(sort_order);
        self.music_catalogue = MusicCatalogue::new();

        for i in start_index..end_index {
            let track_index = self
                .track_list
                .fetch_track_index(&mut self.fs, i as u32)
                .await?;
            let track = self
                .music_catalogue
                .fetch_track(&mut self.fs, track_index)
                .await?;
            playlist.add_back(track);
        }

        Ok(playlist)
    }
}
