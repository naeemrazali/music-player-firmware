#[derive(Copy, Clone)]
pub enum Button {
    Play,
    Menu,
    Next,
    Prev,
}

pub enum Event {
    ButtonPress(Button),
    Seek(u32),
}
