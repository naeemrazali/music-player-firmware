use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::DrawTarget;

use crate::screen_names::ScreenName;
use crate::screens;

pub struct ScreenManager {
    pub main: screens::MainScreen,
    pub settings: screens::SettingsScreen,
    current_screen: ScreenName,
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self {
            main: screens::MainScreen::default(),
            settings: screens::SettingsScreen::default(),
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
