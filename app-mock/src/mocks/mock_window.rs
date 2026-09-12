use app_core::ui::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use embedded_graphics_simulator::{OutputSettingsBuilder, Window};

pub fn new() -> Window {
    let output_settings = OutputSettingsBuilder::new()
        .scale(PIXEL_SCALE)
        .pixel_spacing(PIXEL_SPACING)
        .build();

    Window::new(
        &format!(
            "Audio Player Simulator — {DISPLAY_WIDTH}×{DISPLAY_HEIGHT} ({PIXEL_SCALE}× scale)"
        ),
        &output_settings,
    )
}
