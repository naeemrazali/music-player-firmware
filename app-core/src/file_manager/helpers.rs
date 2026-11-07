pub const U32_BYTES: usize = 4;
pub const U64_BYTES: usize = 8;

pub type U32Buf = [u8; U32_BYTES];
pub type U64Buf = [u8; U64_BYTES];

pub fn parse_u32_le(buf: &[u8], offset: usize) -> u32 {
    let mut bytes = U32Buf::default();
    bytes.copy_from_slice(&buf[offset..offset + U32_BYTES]);
    u32::from_le_bytes(bytes)
}

pub fn parse_u64_le(buf: &[u8], offset: usize) -> u64 {
    let mut bytes = U64Buf::default();
    bytes.copy_from_slice(&buf[offset..offset + U64_BYTES]);
    u64::from_le_bytes(bytes)
}
