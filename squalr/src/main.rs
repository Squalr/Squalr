mod app;
mod ui;

use app::App;
use eframe::NativeOptions;
use eframe::egui::{IconData, ViewportBuilder};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;

static ICON_APP: &[u8] = include_bytes!("../images/app/app_icon.png");

pub fn main() {
    // Create a standalone engine (same process for gui and engine).
    let mut squalr_engine = match SqualrEngine::new(EngineMode::Standalone) {
        Ok(squalr_engine) => squalr_engine,
        Err(error) => panic!("Fatal error initializing Squalr engine: {}", error),
    };

    let icon = image::load_from_memory(ICON_APP)
        .unwrap_or_default()
        .into_rgba8();
    let icon_width = icon.width();
    let icon_height = icon.height();

    // Register app icon, set window size, and disable default window border so that we can add a custom one.
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData {
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            })
            .with_inner_size([1280.0, 840.0])
            .with_min_inner_size([512.0, 256.0])
            .with_decorations(false)
            .with_transparent(true),
        ..NativeOptions::default()
    };

    // Run the gui.
    match eframe::run_native(
        "Squalr",
        native_options,
        Box::new(|creation_context| {
            let app = App::new(&creation_context.egui_ctx, squalr_engine.get_dependency_container());

            // Now that gui dependencies are registered, start the engine fully.
            squalr_engine.initialize();

            Ok(Box::new(app))
        }),
    ) {
        Ok(_) => {}
        Err(error) => {
            panic!("Fatal error in Squalr event loop: {}", error);
        }
    }
}
