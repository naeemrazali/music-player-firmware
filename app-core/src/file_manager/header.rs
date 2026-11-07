pub const HEADER_SIZE_BYTES: usize = 24;

const U32_BYTES: usize = 4;
const U64_BYTES: usize = 8;

pub type U32Buf = [u8; U32_BYTES];
pub type U64Buf = [u8; U64_BYTES];

#[derive(Default)]
pub struct Header {
    magic_number: u32,
    version_number: u32,
    num_items: u32,
    hash: u64,
    reserved: u32,
}

impl Header {
    pub fn parse(buf: &[u8]) -> Self {
        let mut header = Header::default();
        let mut byte_offset = 0;
        header.magic_number = parse_u32_le(buf, byte_offset);
        byte_offset += U32_BYTES;
        header.version_number = parse_u32_le(buf, byte_offset);
        byte_offset += U32_BYTES;
        header.num_items = parse_u32_le(buf, byte_offset);
        byte_offset += U32_BYTES;
        header.hash = parse_u64_le(buf, byte_offset);
        header
    }

    pub fn num_items(&self) -> u32 {
        self.num_items
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
