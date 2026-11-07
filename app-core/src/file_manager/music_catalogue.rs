use exfat_slim::asynchronous::BlockDevice;
use heapless::{String, Vec};

use crate::{
    file_manager::{
        filesystem_io::{Fs, FsError, SD_BLOCK_SIZE, open_read, read_at},
        header::{HEADER_SIZE_BYTES, Header, HeaderBuf},
        helpers::parse_u32_le,
    },
    track::{MAX_STRING_LENGTH, Track},
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
        let mut entry = [0u8; ENTRY_SIZE_BYTES];

        read_at(fs, &mut file, offset, &mut entry).await?;

        let track = Track {
            title: self.parse_title(&entry),
            artist: self.parse_artist(&entry),
            duration_ms: self.parse_duration(&entry),
            index: index as usize,
        };

        Ok(track)
    }

    fn parse_title(&self, buf: &[u8]) -> String<MAX_STRING_LENGTH> {
        // TODO(errors): check if buf is long enough
        let end = buf[..MAX_STRING_LENGTH]
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(MAX_STRING_LENGTH);
        String::from_utf8(Vec::from_slice(&buf[..end]).unwrap_or_default()).unwrap_or_default()
    }

    fn parse_artist(&self, buf: &[u8]) -> String<MAX_STRING_LENGTH> {
        // TODO(errors): check if buf is long enough
        let end = buf[MAX_STRING_LENGTH..(MAX_STRING_LENGTH * 2)]
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(MAX_STRING_LENGTH);
        String::from_utf8(Vec::from_slice(&buf[..end]).unwrap_or_default()).unwrap_or_default()
    }

    fn parse_duration(&self, buf: &[u8]) -> u32 {
        // TODO(refactor): make this constant better?
        const OFFSET: usize = (MAX_STRING_LENGTH * 2) + 64usize;
        parse_u32_le(buf, OFFSET)
    }
}
