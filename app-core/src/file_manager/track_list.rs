use exfat_slim::asynchronous::BlockDevice;
use exfat_slim::asynchronous::file::File;

use crate::file_manager::helpers::U32Buf;
use crate::{
    file_manager::{
        filesystem_io::{Fs, FsError, SD_BLOCK_SIZE, open_read, read_at},
        header::{HEADER_SIZE_BYTES, Header, HeaderBuf},
    },
    playlist::SortOrder,
};

const ENTRY_SIZE_BYTES: usize = 4;

#[derive(Default)]
pub struct TrackList {
    sort_order: SortOrder,
    filepath: &'static str,
    file: Option<File>,
    header: Option<Header>,
}

impl TrackList {
    pub async fn new<D: BlockDevice<SD_BLOCK_SIZE>>(fs: &mut Fs<D>, sort_order: SortOrder) -> Self {
        let mut track_list = Self {
            sort_order,
            filepath: match sort_order {
                SortOrder::TitleAscending | SortOrder::TitleDescending => "\\title_asc.LST",
                SortOrder::ArtistAscending | SortOrder::ArtistDescending => "\\artist_asc.LST",
            },
            ..Default::default()
        };

        track_list.open(fs).await;
        track_list
    }

    pub async fn fetch_track_index<D: BlockDevice<SD_BLOCK_SIZE>>(
        &mut self,
        fs: &mut Fs<D>,
        index: u32,
    ) -> Result<u32, FsError> {
        if self.is_none() | (index >= self.num_items()) {
            return Err(FsError::Io);
        }

        let index = self.convert_to_actual(index);

        if let Some(file) = self.file.as_mut() {
            let offset = (HEADER_SIZE_BYTES + (ENTRY_SIZE_BYTES * index as usize)) as u64;
            let mut entry = U32Buf::default();

            read_at(fs, file, offset, &mut entry).await?;
            Ok(u32::from_le_bytes(entry))
        } else {
            Err(FsError::Io)
        }
    }

    async fn open<D: BlockDevice<SD_BLOCK_SIZE>>(&mut self, fs: &mut Fs<D>) {
        self.file = open_read(fs, self.filepath).await.ok();

        if let Some(file) = self.file.as_mut() {
            let mut buf = HeaderBuf::default();
            // TODO(errors): validate magic "MLST" + version here (stale-generation detection)
            match read_at(fs, file, 0, &mut buf).await {
                Ok(_) => self.header = Some(Header::parse(&buf)),
                Err(_) => self.header = None,
            }
        }
    }

    fn is_none(&self) -> bool {
        self.file.is_none() | self.header.is_none()
    }

    fn num_items(&self) -> u32 {
        if let Some(header) = self.header.as_ref() {
            header.num_items()
        } else {
            u32::default()
        }
    }

    fn convert_to_actual(&self, index: u32) -> u32 {
        // TODO(errors): check return values for integer errors?
        match self.sort_order {
            SortOrder::TitleAscending | SortOrder::ArtistAscending => index,
            SortOrder::TitleDescending | SortOrder::ArtistDescending => {
                self.num_items() - 1 - index
            }
        }
    }
}
