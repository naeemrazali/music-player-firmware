use heapless::String;

pub const MAX_STRING_LENGTH: usize = 256;

#[derive(Default, Clone)]
pub struct Track {
    pub title: String<MAX_STRING_LENGTH>,
    pub artist: String<MAX_STRING_LENGTH>,
    pub duration_ms: u32,
    pub index: usize,
}
