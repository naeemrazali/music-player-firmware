use app_core::file_manager::{header::HEADER_SIZE_BYTES, helpers::U32_BYTES};

const NUM_TRACKS: usize = 4;
const MOCK_TRACK_LIST_BYTES: usize = HEADER_SIZE_BYTES + (U32_BYTES * NUM_TRACKS);

pub fn new() -> [u8; MOCK_TRACK_LIST_BYTES] {
    let mut track_list = [0u8; MOCK_TRACK_LIST_BYTES];

    let magic_number: u32 = 67;
    let version_number: u32 = 1;
    let num_items: u32 = NUM_TRACKS as u32;
    let hash: u32 = 1234;
    let reserved: u32 = 0;
    let country_roads_index: u32 = 0;
    let idol_index: u32 = 1;
    let dancing_queen_index: u32 = 2;
    let invisible_index: u32 = 3;

    track_list[0..4].copy_from_slice(&magic_number.to_le_bytes());
    track_list[4..8].copy_from_slice(&version_number.to_le_bytes());
    track_list[8..12].copy_from_slice(&num_items.to_le_bytes());
    track_list[12..20].copy_from_slice(&hash.to_le_bytes());
    track_list[20..24].copy_from_slice(&reserved.to_le_bytes());
    track_list[24..28].copy_from_slice(&country_roads_index.to_le_bytes());
    track_list[28..32].copy_from_slice(&dancing_queen_index.to_le_bytes());
    track_list[32..36].copy_from_slice(&idol_index.to_le_bytes());
    track_list[36..40].copy_from_slice(&invisible_index.to_le_bytes());

    track_list
}
