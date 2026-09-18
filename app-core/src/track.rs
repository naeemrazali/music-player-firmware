use heapless::String;

const MAX_PATH_LENGTH: usize = 128;
pub const MAX_STRING_LENGTH: usize = 32;

#[derive(Clone, Default)]
pub struct Track {
    pub title: String<MAX_STRING_LENGTH>,
    pub artist: String<MAX_STRING_LENGTH>,
    pub duration_ms: u32,
    pub file_path: String<MAX_PATH_LENGTH>,
}
