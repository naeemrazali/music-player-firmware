use heapless::String;

pub const MAX_STRING_LENGTH: usize = 32;

#[derive(Clone, Default)]
pub struct Track {
    pub title: String<MAX_STRING_LENGTH>,
    pub artist: String<MAX_STRING_LENGTH>,
    pub duration_ms: u32,
    pub file_index: usize,
}
