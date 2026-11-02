use crate::playlist::SortOrder;

#[derive(Default)]
struct SortedList {
    sort_order: SortOrder,
    filepath: &'static str,
}

impl SortedList {
    pub fn new(sort_order: SortOrder) -> Self {
        Self {
            sort_order,
            filepath: match sort_order {
                SortOrder::TitleAscending | SortOrder::TitleDescending => "TASC.LST",
                SortOrder::ArtistAscending | SortOrder::ArtistDescending => "AASC.LST",
            },
        }
    }
}
