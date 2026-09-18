use core::str::FromStr;
use heapless::String;

use app_core::{player::Player, playlist::Playlist, track::Track};

pub fn new() -> Player {
    let mut playlist = Playlist::new();
    let _ = playlist.add(Track {
        title: String::from_str("Clair de Lune").unwrap(),
        artist: String::from_str("Claude Debussy").unwrap(),
        duration_ms: 354000,
        file_index: 0,
    });
    let _ = playlist.add(Track {
        title: String::from_str("Gymnopédie No.1").unwrap(),
        artist: String::from_str("Erik Satie").unwrap(),
        duration_ms: 210000,
        file_index: 1,
    });
    Player::new(playlist)
}
