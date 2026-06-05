use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::DrawTarget;

use crate::player::Player;
use crate::ui::event::Event;
use crate::ui::screen_names::ScreenName;
use crate::ui::screens::main_screen::MainScreen;
use crate::ui::screens::settings_screen::SettingsScreen;

pub struct ScreenManager {
    pub main: MainScreen,
    pub settings: SettingsScreen,
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

impl ScreenManager {
    pub fn handle_event(&mut self, event: &Event, player: &mut Player) {
        let next_screen = match self.current_screen {
            ScreenName::Main => self.main.handle_event(event, player),
            ScreenName::Settings => self.settings.handle_event(event, player),
        };

        if let Some(new_screen) = next_screen {
            self.current_screen = new_screen;
        }
    }

    pub fn sync(&mut self, player: &Player) {
        match self.current_screen {
            ScreenName::Main => self.main.sync(player),
            ScreenName::Settings => self.settings.sync(player),
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
