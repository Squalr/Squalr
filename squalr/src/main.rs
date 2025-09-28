mod app;
mod ui;

use app::App;
use eframe::NativeOptions;
use eframe::egui::ViewportBuilder;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;

pub fn main() {
    // Create a standalone engine (same process for gui and engine).
    let mut squalr_engine = match SqualrEngine::new(EngineMode::Standalone) {
        Ok(squalr_engine) => squalr_engine,
        Err(error) => panic!("Fatal error initializing Squalr engine: {}", error),
    };

    // Disable default window border so that we can add a custom one.
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1280.0, 840.0])
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
