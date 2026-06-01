#[derive(Copy, Clone)]
pub enum Button {
    Play,
    Menu,
}

#[derive(Copy, Clone)]
pub enum Action {
    Pressed,
}

pub struct Event {
    pub button: Button,
    pub action: Action,
}
