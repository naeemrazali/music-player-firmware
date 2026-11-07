use core::str::FromStr;
use heapless::String;

use app_core::{player::Player, playlist::Playlist, track::Track};

pub fn new() -> Player {
    let mut playlist = Playlist::new();
    playlist.add_back(Track {
        title: String::from_str("Clair de Lune").unwrap(),
        artist: String::from_str("Claude Debussy").unwrap(),
        duration_ms: 354000,
        index: 0usize,
    });
    playlist.add_back(Track {
        title: String::from_str("Gymnopédie No.1").unwrap(),
        artist: String::from_str("Erik Satie").unwrap(),
        duration_ms: 210000,
        index: 0usize,
    });
    Player::new(playlist)
}
