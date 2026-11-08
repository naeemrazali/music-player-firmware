use exfat_slim::asynchronous::{BlockDevice, file::File};
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

#[derive(Default)]
pub struct MusicCatalogue {
    filepath: &'static str,
    file: Option<File>,
    header: Option<Header>,
}

impl MusicCatalogue {
    pub async fn new<D: BlockDevice<SD_BLOCK_SIZE>>(fs: &mut Fs<D>) -> Self {
        let mut catalogue = Self {
            filepath: "\\music.cat",
            ..Default::default()
        };
        catalogue.open(fs).await;
        catalogue
    }

    pub async fn fetch_track<D: BlockDevice<SD_BLOCK_SIZE>>(
        &mut self,
        fs: &mut Fs<D>,
        index: u32,
    ) -> Result<Track, FsError> {
        if self.is_none() | (index >= self.num_items()) {
            return Err(FsError::Io);
        }

        if let Some(file) = self.file.as_mut() {
            let offset = (HEADER_SIZE_BYTES + (ENTRY_SIZE_BYTES * index as usize)) as u64;
            let mut entry = [0u8; ENTRY_SIZE_BYTES];
            read_at(fs, file, offset, &mut entry).await?;

            let track = Track {
                title: self.parse_title(&entry),
                artist: self.parse_artist(&entry),
                duration_ms: self.parse_duration(&entry),
                index: index as usize,
            };
            Ok(track)
        } else {
            // TODO(errors): better error code?
            Err(FsError::Io)
        }
    }

    // TODO(refactor): make into a shared trait with other custom file type readers
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

    // TODO(refactor): make into a shared trait with other custom file type readers
    fn is_none(&self) -> bool {
        self.file.is_none() | self.header.is_none()
    }

    // TODO(refactor): make into a shared trait with other custom file type readers
    fn num_items(&self) -> u32 {
        if let Some(header) = self.header.as_ref() {
            header.num_items()
        } else {
            u32::default()
        }
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
