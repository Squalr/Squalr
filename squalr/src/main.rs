mod app;

use app::MyApp;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
// use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

pub fn main() {
    // Create a standalone engine (same process for gui and engine).
    let mut squalr_engine = match SqualrEngine::new(EngineMode::Standalone) {
        Ok(squalr_engine) => squalr_engine,
        Err(error) => panic!("Fatal error initializing Squalr engine: {}", error),
    };

    // Create and show the main window, which in turn will instantiate all dockable windows.
    // May not evaluate until the dependencies in the engine are initialized.
    // MainWindowViewModel::register(squalr_engine.get_dependency_container());

    // Now that gui dependencies are registered, start the engine fully.
    squalr_engine.initialize();

    let app = MyApp::default();
    let native_options = eframe::NativeOptions::default();

    match eframe::run_native("Squalr", native_options, Box::new(|_creation_context| Ok(Box::new(app)))) {
        Ok(_) => {}
        Err(error) => {
            panic!("Fatal error in Squalr event loop: {}", error);
        }
    }
}
