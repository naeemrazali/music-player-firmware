use crate::file_manager::helpers::{U32_BYTES, U64_BYTES, parse_u32_le, parse_u64_le};

pub const HEADER_SIZE_BYTES: usize = 24;

pub type HeaderBuf = [u8; HEADER_SIZE_BYTES];

#[derive(Default)]
pub struct Header {
    magic_number: u32,
    version_number: u32,
    num_items: u32,
    hash: u64,
    reserved: u32,
}

impl Header {
    // TODO(errors): parsing could fail, handle failure gracefully in future implementation
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
        byte_offset += U64_BYTES;
        header.reserved = parse_u32_le(buf, byte_offset);
        header
    }

    pub fn to_le_bytes(&self) -> [u8; HEADER_SIZE_BYTES] {
        let mut buf = [0u8; HEADER_SIZE_BYTES];
        buf[0..4].copy_from_slice(&self.magic_number.to_le_bytes());
        buf[4..8].copy_from_slice(&self.version_number.to_le_bytes());
        buf[8..12].copy_from_slice(&self.num_items.to_le_bytes());
        buf[12..20].copy_from_slice(&self.hash.to_le_bytes());
        buf[20..24].copy_from_slice(&self.reserved.to_le_bytes());
        buf
    }

    pub fn num_items(&self) -> u32 {
        self.num_items
    }
}
