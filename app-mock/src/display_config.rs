/// The target display resolution in pixels.
///
/// Set this to match your chosen embedded display once you have selected one.
/// Common values:
///   240 x 240  — 1.3" / 1.54" round or square TFT (e.g. GC9A01, ST7789)
///   320 x 240  — 2.0" / 2.4" TFT (e.g. ILI9341)
///   128 x 160  — 1.8" TFT (e.g. ST7735)
///   480 x 320  — 3.5" TFT (e.g. ILI9488)
pub const DISPLAY_WIDTH:  u32 = 240;
pub const DISPLAY_HEIGHT: u32 = 240;

/// Scale factor applied to every pixel when rendering on the desktop.
///
/// The physical display is small; scaling up makes it comfortable to
/// view and interact with during development. Does not affect any pixel
/// coordinates in your drawing code — those always use the values above.
pub const PIXEL_SCALE: u32 = 3;

/// Gap (in desktop pixels) rendered between each simulated pixel.
/// Set to 0 for a solid display look; 1 gives a faint grid that mimics
/// the pixel gaps visible on real LCD panels.
pub const PIXEL_SPACING: u32 = 0;
