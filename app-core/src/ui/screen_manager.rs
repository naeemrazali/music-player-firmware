use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;

use crate::event::{Event, Screen};
use crate::ui::screen_names::ScreenName;
use crate::ui::screens::main_screen::MainScreen;
use crate::ui::screens::settings_screen::SettingsScreen;

pub struct ScreenManager {
    main: MainScreen,
    settings: SettingsScreen,
    current_screen: ScreenName,
    pub needs_refresh: bool,
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self {
            main: MainScreen::default(),
            settings: SettingsScreen::default(),
            current_screen: ScreenName::Main,
            needs_refresh: true,
        }
    }
}

impl ScreenManager {
    pub fn handle_event(&mut self, event: &Event) {
        if let Event::Ui(Screen::Change(screen)) = event {
            self.current_screen = *screen;
        } else if let Event::Ui(Screen::Refresh) = event {
            self.needs_refresh = true;
        }
        match self.current_screen {
            ScreenName::Main => self.main.handle_event(event),
            ScreenName::Settings => self.settings.handle_event(event),
        }
    }

    pub fn event_queue(&mut self) -> heapless::Vec<Event, 8> {
        match self.current_screen {
            ScreenName::Main => self.main.event_queue(),
            ScreenName::Settings => self.settings.event_queue(),
        }
    }

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
