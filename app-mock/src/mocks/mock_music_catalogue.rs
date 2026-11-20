use app_core::{file_manager::header::HEADER_SIZE_BYTES, track::MAX_STRING_LENGTH};

const ENTRY_SIZE_BYTES: usize = 580;
const NUM_ENTRIES: usize = 4;
const MOCK_MUSIC_CAT_BYTES: usize = HEADER_SIZE_BYTES + (ENTRY_SIZE_BYTES * NUM_ENTRIES);

pub fn new() -> [u8; MOCK_MUSIC_CAT_BYTES] {
    let mut track_list = [0u8; MOCK_MUSIC_CAT_BYTES];

    let magic_number: u32 = 67;
    let version_number: u32 = 1;
    let num_items: u32 = NUM_ENTRIES as u32;
    let hash: u64 = 1234;
    let reserved: u32 = 0;

    let country_roads_title = null_padded_utf8_buf("Country Roads");
    let country_roads_artist = null_padded_utf8_buf("John Denver");
    let country_roads_duration: u32 = 190;

    let idol_title = null_padded_utf8_buf("Idol");
    let idol_artist = null_padded_utf8_buf("YOASOBI");
    let idol_duration: u32 = 213;

    let dancing_queen_title = null_padded_utf8_buf("Dancing Queen");
    let dancing_queen_artist = null_padded_utf8_buf("ABBA");
    let dancing_queen_duration: u32 = 230;

    let invisible_title = null_padded_utf8_buf("Invisible");
    let invisible_artist = null_padded_utf8_buf("Duran Duran");
    let invisible_duration: u32 = 191;

    let file_path = [0u8; 64];

    track_list[0..4].copy_from_slice(&magic_number.to_le_bytes());
    track_list[4..8].copy_from_slice(&version_number.to_le_bytes());
    track_list[8..12].copy_from_slice(&num_items.to_le_bytes());
    track_list[12..20].copy_from_slice(&hash.to_le_bytes());
    track_list[20..24].copy_from_slice(&reserved.to_le_bytes());

    track_list[24..280].copy_from_slice(&country_roads_title);
    track_list[280..536].copy_from_slice(&country_roads_artist);
    track_list[536..600].copy_from_slice(&file_path);
    track_list[600..604].copy_from_slice(&country_roads_duration.to_le_bytes());

    track_list[604..860].copy_from_slice(&idol_title);
    track_list[860..1116].copy_from_slice(&idol_artist);
    track_list[1116..1180].copy_from_slice(&file_path);
    track_list[1180..1184].copy_from_slice(&idol_duration.to_le_bytes());

    track_list[1184..1440].copy_from_slice(&dancing_queen_title);
    track_list[1440..1696].copy_from_slice(&dancing_queen_artist);
    track_list[1696..1760].copy_from_slice(&file_path);
    track_list[1760..1764].copy_from_slice(&dancing_queen_duration.to_le_bytes());

    track_list[1764..2020].copy_from_slice(&invisible_title);
    track_list[2020..2276].copy_from_slice(&invisible_artist);
    track_list[2276..2340].copy_from_slice(&file_path);
    track_list[2340..2344].copy_from_slice(&invisible_duration.to_le_bytes());

    track_list
}

fn null_padded_utf8_buf(s: &str) -> [u8; MAX_STRING_LENGTH] {
    let bytes = s.as_bytes();
    let mut buf = [0u8; MAX_STRING_LENGTH];
    buf[..bytes.len()].copy_from_slice(bytes);
    buf
}
