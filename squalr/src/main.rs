// Disable terminal from spawning. All relevant output is routed to the output view anyways.
#![windows_subsystem = "windows"]

use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

pub fn main() {
    // Override the default backend (femtovg => software). I wont want to rely on devs having QT installed,
    // and femtovg is shit at rendering font. Skia doesn't install properly on nightly, so software renderer it is.
    unsafe {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }

    let squalr_engine = SqualrEngine::new(EngineMode::Standalone);

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = MainWindowViewModel::new(squalr_engine.get_engine_execution_context(), squalr_engine.get_logger());

    // Start the log event sending now that both the GUI and engine are ready to receive log messages.
    squalr_engine.get_logger().start_log_event_sender();

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(err) => {
            log::error!("Fatal error starting Squalr: {}", err);
        }
    }
}
