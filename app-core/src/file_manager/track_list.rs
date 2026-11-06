use exfat_slim::asynchronous::{BlockDevice, file_system::FileSystem};

use crate::{
    file_manager::filesystem_io::{FsError, open_read, read_at},
    playlist::SortOrder,
};

const HEADER_SIZE_BYTES: usize = 24;
const ENTRY_SIZE_BYTES: usize = 4;
const U32_BYTES: usize = 4;
const U64_BYTES: usize = 8;

type U32Buf = [u8; U32_BYTES];
type U64Buf = [u8; U64_BYTES];

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
                SortOrder::TitleAscending | SortOrder::TitleDescending => "\\TASC.LST",
                SortOrder::ArtistAscending | SortOrder::ArtistDescending => "\\AASC.LST",
            },
        }
    }

    pub async fn fetch_track_index<D: BlockDevice<512>, const N: usize>(
        &mut self,
        fs: &mut FileSystem<D, 512, N>,
        index: u32,
    ) -> Result<u32, FsError> {
        let mut file = open_read(fs, self.filepath).await?;
        let mut header = Header::new();

        read_at(fs, &mut file, 0, &mut header.raw_buffer).await?;
        header.parse();
        // TODO(errors): validate magic "MLST" + version here (stale-generation detection)

        if index >= header.num_items {
            // TODO(errors): dedicated OutOfRange variant (currently maps to Io)
            return Err(FsError::Io);
        }

        let offset = HEADER_SIZE_BYTES as u64 + (ENTRY_SIZE_BYTES as u64 * index as u64);
        let mut entry = U32Buf::default();

        read_at(fs, &mut file, offset, &mut entry).await?;
        Ok(u32::from_le_bytes(entry))
    }
}

#[derive(Default)]
struct Header {
    raw_buffer: [u8; HEADER_SIZE_BYTES],
    magic_number: u32,
    version_number: u32,
    num_items: u32,
    hash: u64,
    reserved: u32,
}

impl Header {
    fn new() -> Self {
        Self {
            raw_buffer: [0u8; HEADER_SIZE_BYTES],
            ..Default::default()
        }
    }

    fn parse(&mut self) {
        let mut byte_offset = 0;
        self.magic_number = parse_u32_le(&self.raw_buffer, byte_offset);
        byte_offset += U32_BYTES;
        self.version_number = parse_u32_le(&self.raw_buffer, byte_offset);
        byte_offset += U32_BYTES;
        self.num_items = parse_u32_le(&self.raw_buffer, byte_offset);
        byte_offset += U32_BYTES;
        self.hash = parse_u64_le(&self.raw_buffer, byte_offset);
    }
}

fn parse_u32_le(buf: &[u8], offset: usize) -> u32 {
    let mut bytes = U32Buf::default();
    bytes.copy_from_slice(&buf[offset..offset + U32_BYTES]);
    u32::from_le_bytes(bytes)
}

fn parse_u64_le(buf: &[u8], offset: usize) -> u64 {
    let mut bytes = U64Buf::default();
    bytes.copy_from_slice(&buf[offset..offset + U64_BYTES]);
    u64::from_le_bytes(bytes)
}
