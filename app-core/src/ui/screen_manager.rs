use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;

use crate::event::{Event, EventHandler, EventQueue, Screen};
use crate::ui::screen_names::ScreenName;
use crate::ui::screens::main_screen::MainScreen;
use crate::ui::screens::settings_screen::SettingsScreen;

pub struct ScreenManager {
    main: MainScreen,
    settings: SettingsScreen,
    current_screen: ScreenName,
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self {
            main: MainScreen::default(),
            settings: SettingsScreen::default(),
            current_screen: ScreenName::Main,
        }
    }
}

impl EventHandler for ScreenManager {
    fn event_queue(&mut self) -> &mut EventQueue {
        match self.current_screen {
            ScreenName::Main => self.main.event_queue(),
            ScreenName::Settings => self.settings.event_queue(),
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventQueue {
        if let Event::Ui(Screen::Change(screen)) = event {
            self.current_screen = *screen;
            return heapless::Vec::new();
        }
        match self.current_screen {
            ScreenName::Main => self.main.handle_event(event),
            ScreenName::Settings => self.settings.handle_event(event),
        }
    }
}

impl ScreenManager {
    pub fn draw(
        &self,
        display: &mut impl DrawTarget<Color = Gray8, Error = impl core::fmt::Debug>,
    ) {
        match self.current_screen {
            ScreenName::Main => self.main.draw(display),
            ScreenName::Settings => self.settings.draw(display),
        }
    }
}
