use app_core::ui::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embedded_graphics::{pixelcolor::Gray8, prelude::Size};
use embedded_graphics_simulator::SimulatorDisplay;

pub type MockSimulatorDisplay = SimulatorDisplay<Gray8>;

pub fn new() -> MockSimulatorDisplay {
    SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT))
}
