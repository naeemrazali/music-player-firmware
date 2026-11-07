use exfat_slim::asynchronous::BlockDevice;

use crate::{
    file_manager::{
        filesystem_io::{Fs, FsError, SD_BLOCK_SIZE, open_read, read_at},
        header::{HEADER_SIZE_BYTES, Header, HeaderBuf},
    },
    track::Track,
};

const ENTRY_SIZE_BYTES: usize = 580;

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
        let mut header_buf = HeaderBuf::default();

        read_at(fs, &mut file, 0, &mut header_buf).await?;
        let header = Header::parse(&header_buf);

        if index >= header.num_items() {
            return Err(FsError::Io);
        }

        let offset = HEADER_SIZE_BYTES as u64 + (ENTRY_SIZE_BYTES as u64 * index as u64);
        let mut entry = 
    }
}
