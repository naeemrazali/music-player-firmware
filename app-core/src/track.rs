use heapless::String;

const MAX_PATH_LENGTH: usize = 128;
const MAX_STR_LENGTH: usize = 64;

#[derive(Clone, Default)]
pub struct Track {
    pub title: String<MAX_STR_LENGTH>,
    pub artist: String<MAX_STR_LENGTH>,
    pub duration_ms: u32,
    pub file_path: String<MAX_PATH_LENGTH>,
}
