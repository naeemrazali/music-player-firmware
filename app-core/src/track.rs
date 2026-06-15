#[derive(Copy, Clone)]
pub struct Track {
    pub title: &'static str,
    pub artist: &'static str,
    pub duration_ms: u32,
    pub file_path: &'static str,
}
