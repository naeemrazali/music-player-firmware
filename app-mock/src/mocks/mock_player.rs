use app_core::{player::Player, playlist::Playlist, track::Track};

pub fn new() -> Player {
    let mut playlist = Playlist::new();
    let _ = playlist.add(Track {
        title: "Clair de Lune",
        artist: "Claude Debussy",
        duration_ms: 354000,
        file_path: "/music/flac/clair_de_lune.flac",
    });
    let _ = playlist.add(Track {
        title: "Gymnopédie No.1",
        artist: "Erik Satie",
        duration_ms: 210000,
        file_path: "/music/flac/gymnopedie_no1.flac",
    });
    Player::new(playlist)
}
