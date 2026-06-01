use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::DrawTarget;

use crate::main_screen::MainScreen;
use crate::screen_names::ScreenName;
use crate::settings_screen::SettingsScreen;

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
    // change screen function (according to enum)
    pub fn change_screen(&mut self, screen: ScreenName) {
        self.current_screen = screen;
    }

    // toggle between main and settings screen as a placeholder for now
    pub fn toggle_screens(&mut self) {
        if matches!(self.current_screen, ScreenName::Main) {
            self.current_screen = ScreenName::Settings;
        } else {
            self.current_screen = ScreenName::Main;
        }
    }

    // draw function
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
