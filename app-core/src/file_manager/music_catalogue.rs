use exfat_slim::asynchronous::BlockDevice;

use crate::{
    file_manager::filesystem_io::{Fs, FsError, SD_BLOCK_SIZE, open_read},
    track::Track,
};

pub struct MusicCatalogue {
    filepath: &'static str,
}

impl MusicCatalogue {
    pub fn new() -> Self {
        Self {
            filepath: "\\music.cat",
        }
    }

    pub async fn fetch_track<D: BlockDevice<SD_BLOCK_SIZE>>(
        &self,
        fs: &mut Fs<D>,
        index: u32,
    ) -> Result<Track, FsError> {
        let mut file = open_read(fs, self.filepath).await?;
        let mut
    }
}
