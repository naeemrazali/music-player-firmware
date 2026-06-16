#[derive(Copy, Clone)]
pub struct Track {
    pub title: &'static str,
    pub artist: &'static str,
    pub duration_ms: u32,
    pub file_path: &'static str,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            title: "",
            artist: "",
            duration_ms: 0,
            file_path: "",
        }
    }
}
