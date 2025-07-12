// Disable terminal from spawning. All relevant output is routed to the output view anyways.
#![windows_subsystem = "windows"]

use olorin_engine::engine_mode::EngineMode;
use olorin_engine::olorin_engine::OlorinEngine;
use olorin_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

pub fn main() {
    // Override the default rendering backend (femtovg => software). I wont want to rely on devs having QT installed,
    // and femtovg is shit at rendering font. Skia doesn't install properly on nightly, so software renderer it is.
    unsafe {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }

    // Create a standalone engine (same process for gui and engine).
    let mut olorin_engine = match OlorinEngine::new(EngineMode::Standalone) {
        Ok(olorin_engine) => olorin_engine,
        Err(error) => panic!("Fatal error initializing Olorin engine: {}", error),
    };

    // Create and show the main window, which in turn will instantiate all dockable windows.
    // May not evaluate until the dependencies in the engine are initialized.
    MainWindowViewModel::register(olorin_engine.get_dependency_container());

    // Now that gui dependencies are registered, start the engine fully.
    olorin_engine.initialize();

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(error) => {
            panic!("Fatal error in Olorin event loop: {}", error);
        }
    }
}
