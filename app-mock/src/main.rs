use app_core::display_config::{DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_SCALE, PIXEL_SPACING};
use app_core::screens::{AppScreen, MainScreen, SettingsScreen};
use app_core::ui::Widget;
use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

fn main() {
    let mut display: SimulatorDisplay<Gray8> =
        SimulatorDisplay::new(Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));

    let output_settings = OutputSettingsBuilder::new()
        .scale(PIXEL_SCALE)
        .pixel_spacing(PIXEL_SPACING)
        .build();

    let mut window = Window::new(
        &format!(
            "Audio Player Simulator  —  {DISPLAY_WIDTH}×{DISPLAY_HEIGHT}  ({PIXEL_SCALE}× scale)"
        ),
        &output_settings,
    );

    // Mutable state
    let mut is_playing = true;
    let mut elapsed_ms = 0u32;
    let mut on_main_screen = true;

    // Pre-construct both screens
    let mut main_screen = MainScreen {
        background: Gray8::new(0x9F),
        title: Widget::Label(app_core::ui::Label {
            text:     "Clair de Lune",
            position: Point::new(10, 20),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::BLACK),
        }),
        artist: Widget::Label(app_core::ui::Label {
            text:     "Claude Debussy",
            position: Point::new(10, 36),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(100)),
        }),
        progress: Widget::ProgressBar(app_core::ui::ProgressBar {
            rect:       Rectangle::new(Point::new(10, 100), Size::new(220, 6)),
            fg:         Gray8::new(0),
            bg:         Gray8::new(160),
            total_ms:   35400,
            elapsed_ms: 0,
        }),
        play_btn: Widget::PlayButton(app_core::ui::PlayButton {
            center:     Point::new(120, 170),
            color:      Gray8::new(0),
            is_playing: true,
        }),
        elapsed_buf: [0u8; 6],
        total_buf:   [0u8; 6],
    };

    let settings_items: &[&str] = &["Volume", "Shuffle", "Repeat", "Equalizer"];
    let mut settings_screen = SettingsScreen {
        background: Gray8::new(0x9F),
        heading: Widget::Label(app_core::ui::Label {
            text:     "Settings",
            position: Point::new(10, 20),
            style:    MonoTextStyle::new(&FONT_6X10, Gray8::new(0)),
        }),
        separator: Widget::HorizontalLine(app_core::ui::HorizontalLine {
            start: Point::new(10, 34),
            end:   Point::new(230, 34),
            style: PrimitiveStyle::with_stroke(Gray8::new(100), 1),
        }),
        list: Widget::List(app_core::ui::List {
            items:            settings_items,
            selected:         0,
            start:            Point::new(10, 46),
            item_height:      14,
            selected_color:   Gray8::BLACK,
            unselected_color: Gray8::new(100),
            selected_prefix:  "> ",
            unselected_prefix:"  ",
        }),
    };

    loop {
        // ── Update phase ───────────────────────────────────────────────
        if is_playing {
            elapsed_ms += 33;
            if elapsed_ms > 35400 {
                elapsed_ms = 35400;
            }
        }

        // Sync state into main_screen widgets
        main_screen.play_btn = Widget::PlayButton(app_core::ui::PlayButton {
            center:     Point::new(120, 170),
            color:      Gray8::new(0),
            is_playing,
        });
        main_screen.progress = Widget::ProgressBar(app_core::ui::ProgressBar {
            rect:       Rectangle::new(Point::new(10, 100), Size::new(220, 6)),
            fg:         Gray8::new(0),
            bg:         Gray8::new(160),
            total_ms:   35400,
            elapsed_ms,
        });

        // ── Draw phase ───────────────────────────────────────────────────
        let mut current = if on_main_screen {
            AppScreen::Main(&mut main_screen)
        } else {
            AppScreen::Settings(&mut settings_screen)
        };
        current.draw(&mut display);
        window.update(&display);

        // ── Input phase ──────────────────────────────────────────────────
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return,

                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Space => is_playing = !is_playing,
                    Keycode::M     => on_main_screen = !on_main_screen,
                    Keycode::Escape => return,
                    _ => {}
                },

                _ => {}
            }
        }

        // ~30 fps
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
