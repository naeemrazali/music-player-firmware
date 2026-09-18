use super::*;

fn track(title: &'static str, duration_ms: u32) -> Track {
    Track {
        title: title.parse().unwrap(),
        artist: "Test Artist".parse().unwrap(),
        duration_ms,
        file_index: 0,
    }
}

#[test]
fn empty_playlist_has_no_current() {
    let mut p = Playlist::new();
    assert!(p.current().is_none());
    assert!(p.next().is_none());
    assert!(p.prev().is_none());
}

#[test]
fn single_track_next_and_prev_are_none() {
    let mut p = Playlist::new();
    p.add(track("A", 1000)).unwrap();
    assert_eq!(p.current().unwrap().title, "A");
    assert!(p.next().is_none());
    assert_eq!(p.current().unwrap().title, "A"); // stayed at A
    assert!(p.prev().is_none());
    assert_eq!(p.current().unwrap().title, "A"); // stayed at A
}

#[test]
fn multi_track_navigation() {
    let mut p = Playlist::new();
    p.add(track("A", 1000)).unwrap();
    p.add(track("B", 2000)).unwrap();
    p.add(track("C", 3000)).unwrap();

    assert_eq!(p.current().unwrap().title, "A");
    assert_eq!(p.next().unwrap().title, "B");
    assert_eq!(p.next().unwrap().title, "C");
    assert!(p.next().is_none()); // end of playlist
    assert_eq!(p.current().unwrap().title, "C"); // stayed at C

    assert_eq!(p.prev().unwrap().title, "B");
    assert_eq!(p.prev().unwrap().title, "A");
    assert!(p.prev().is_none()); // start of playlist
    assert_eq!(p.current().unwrap().title, "A"); // stayed at A
}
