use super::*;
use crate::track::Track;

fn track(title: &'static str, duration_ms: u32) -> Track {
    Track {
        title: title.parse().unwrap(),
        artist: "Test Artist".parse().unwrap(),
        duration_ms,
        file_index: 0,
    }
}

fn playlist(tracks: &[(&'static str, u32)]) -> Playlist {
    let mut p = Playlist::new();
    for (title, dur) in tracks {
        p.add(track(title, *dur)).unwrap();
    }
    p
}

#[test]
fn starts_playing() {
    let player = Player::new(playlist(&[("A", 1000)]));
    assert!(player.is_playing);
    assert_eq!(player.elapsed_ms, 0);
}

#[test]
fn tick_advances_time() {
    let mut player = Player::new(playlist(&[("A", 5000)]));
    player.tick(1000);
    assert_eq!(player.elapsed_ms, 1000);
    assert!(player.is_playing);
}

#[test]
fn tick_paused_does_nothing() {
    let mut player = Player::new(playlist(&[("A", 5000)]));
    player.toggle_playback();
    player.tick(1000);
    assert_eq!(player.elapsed_ms, 0);
    assert!(!player.is_playing);
}

#[test]
fn tick_at_end_auto_advances() {
    let mut player = Player::new(playlist(&[("A", 1000), ("B", 2000)]));
    player.tick(1000);
    assert_eq!(player.playlist.current().unwrap().title, "B");
    assert_eq!(player.elapsed_ms, 0);
    assert!(player.is_playing);
}

#[test]
fn tick_at_end_of_list_pauses() {
    let mut player = Player::new(playlist(&[("A", 1000)]));
    player.tick(1000);
    assert!(!player.is_playing);
    assert_eq!(player.elapsed_ms, 1000); // stays at end
    assert_eq!(player.playlist.current().unwrap().title, "A"); // no next track
}

#[test]
fn seek_to_changes_elapsed() {
    let mut player = Player::new(playlist(&[("A", 5000)]));
    player.seek_to(40); // 40% of 5000ms = 2000ms
    assert_eq!(player.elapsed_ms, 2000);
}

#[test]
fn seek_to_end_auto_advances() {
    let mut player = Player::new(playlist(&[("A", 1000), ("B", 2000)]));
    player.seek_to(100); // 100% = end of track
    assert_eq!(player.playlist.current().unwrap().title, "B");
    assert_eq!(player.elapsed_ms, 0);
    assert!(player.is_playing);
}

#[test]
fn seek_to_end_of_list_pauses() {
    let mut player = Player::new(playlist(&[("A", 1000)]));
    player.seek_to(100); // 100% = end of track
    assert!(!player.is_playing);
    assert_eq!(player.elapsed_ms, 1000);
}

#[test]
fn next_track_advances_and_resets() {
    let mut player = Player::new(playlist(&[("A", 1000), ("B", 2000)]));
    player.tick(500);
    player.next_track();
    assert_eq!(player.playlist.current().unwrap().title, "B");
    assert_eq!(player.elapsed_ms, 0);
}

#[test]
fn prev_track_goes_back_and_resets() {
    let mut player = Player::new(playlist(&[("A", 1000), ("B", 2000)]));
    player.next_track();
    player.prev_track();
    assert_eq!(player.playlist.current().unwrap().title, "A");
    assert_eq!(player.elapsed_ms, 0);
}

#[test]
fn progress_percent_correct() {
    let mut player = Player::new(playlist(&[("A", 10000)]));
    player.tick(2500);
    let total = player.playlist.current().unwrap().duration_ms;
    assert_eq!((player.elapsed_ms * 100) / total, 25);
}

#[test]
fn toggle_playback() {
    let mut player = Player::new(playlist(&[("A", 1000)]));
    assert!(player.is_playing);
    player.toggle_playback();
    assert!(!player.is_playing);
    player.toggle_playback();
    assert!(player.is_playing);
}
