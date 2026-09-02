use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyle;

use crate::event::{Button, Event, Screen};
use crate::ui;
use crate::ui::screen_names::ScreenName;

pub struct SettingsScreen {
    background: Gray8,
    heading: ui::Label<'static>,
    separator: ui::HorizontalLine,
    list: ui::List<'static>,
    events: heapless::Vec<Event, 8>,
}

impl SettingsScreen {
    pub fn handle_event(&mut self, event: &Event) -> heapless::Vec<Event, 8> {
        if let Event::ButtonPress(Button::Menu) = event {
            self.add_event(Event::Ui(Screen::Change(ScreenName::Main)));
            self.add_event(Event::Ui(Screen::Refresh));
        }
        self.push_events()
    }

    pub fn push_events(&mut self) -> heapless::Vec<Event, 8> {
        let mut events = heapless::Vec::new();
        core::mem::swap(&mut self.events, &mut events);
        events
    }

    fn add_event(&mut self, event: Event) {
        let _ = self.events.push(event);
    }

    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        ui::clear_background(display, self.background);
        self.heading.draw(display);
        self.separator.draw(display);
        self.list.draw(display);
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self {
            background: Gray8::new(0x9F),
            heading: ui::Label::new(
                "Settings",
                Point::new(10, 20),
                MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
            ),
            separator: ui::HorizontalLine::new(
                Point::new(10, 34),
                Point::new(230, 34),
                PrimitiveStyle::with_stroke(Gray8::new(0x64), 1),
            ),
            list: ui::List {
                items: &["Volume", "Shuffle", "Repeat", "Equalizer"],
                selected: 0,
                start: Point::new(10, 46),
                item_height: 14,
                selected_color: Gray8::BLACK,
                unselected_color: Gray8::new(100),
                selected_prefix: "> ",
                unselected_prefix: "  ",
            },
            events: heapless::Vec::new(),
        }
    }
}
