use core::default;

use exfat_slim::asynchronous::BlockDevice;

use crate::{
    file_manager::{
        filesystem_io::{Fs, FsError, SD_BLOCK_SIZE, open_read, read_at},
        header::{HEADER_SIZE_BYTES, Header, HeaderBuf, U32Buf},
    },
    playlist::SortOrder,
};

const ENTRY_SIZE_BYTES: usize = 4;

#[derive(Default)]
pub struct TrackList {
    sort_order: SortOrder,
    filepath: &'static str,
}

impl TrackList {
    pub fn new(sort_order: SortOrder) -> Self {
        Self {
            sort_order,
            filepath: match sort_order {
                SortOrder::TitleAscending | SortOrder::TitleDescending => "\\title_asc.LST",
                SortOrder::ArtistAscending | SortOrder::ArtistDescending => "\\artist_asc.LST",
            },
        }
    }

    pub async fn fetch_track_index<D: BlockDevice<SD_BLOCK_SIZE>>(
        &mut self,
        fs: &mut Fs<D>,
        index: u32,
    ) -> Result<u32, FsError> {
        let mut file = open_read(fs, self.filepath).await?;
        let mut header_buf = HeaderBuf::default();

        read_at(fs, &mut file, 0, &mut header_buf).await?;
        let header = Header::parse(&header_buf);
        // TODO(errors): validate magic "MLST" + version here (stale-generation detection)

        if index >= header.num_items() {
            // TODO(errors): dedicated OutOfRange variant (currently maps to Io)
            return Err(FsError::Io);
        }

        let offset = HEADER_SIZE_BYTES as u64 + (ENTRY_SIZE_BYTES as u64 * index as u64);
        let mut entry = U32Buf::default();

        read_at(fs, &mut file, offset, &mut entry).await?;
        Ok(u32::from_le_bytes(entry))
    }
}
