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

    pub fn num_items(&self) -> u32 {
        self.num_items
    }
}
