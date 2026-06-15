use crate::event::Event;
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
    pub fn handle_event(&mut self, event: &Event) {
        let next_screen = match self.current_screen {
            ScreenName::Main => self.main.handle_event(event),
            ScreenName::Settings => self.settings.handle_event(event),
        };

        if let Some(new_screen) = next_screen {
            self.current_screen = new_screen;
        }
    }
}
