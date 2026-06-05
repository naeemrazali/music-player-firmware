#[derive(Copy, Clone)]
pub enum Button {
    Play,
    Menu,
}

pub enum Event {
    ButtonPress(Button),
    Seek(u32),
}
