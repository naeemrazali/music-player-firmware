use crate::playlist::SortOrder;

#[derive(Default)]
pub struct TrackList {
    sort_order: SortOrder,
    filepath: &'static str,
}

impl TrackList {
    pub fn new(sort_order: SortOrder) -> Self {
        Self {
            sort_order,
            filepath: match sort_order {
                SortOrder::TitleAscending | SortOrder::TitleDescending => "TASC.LST",
                SortOrder::ArtistAscending | SortOrder::ArtistDescending => "AASC.LST",
            },
        }
    }

    pub fn fetch_track(&mut self, index: usize) {
        // open self.filepath
        // check index < num items (from reading the header of the file)
    }
}
