// Disable terminal from spawning. All relevant output is routed to the output view anyways.
#![windows_subsystem = "windows"]

use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

pub fn main() {
    // Override the default backend (femtovg => software). I wont want to rely on devs having QT installed,
    // and femtovg is shit at rendering font. Skia doesn't install properly on nightly, so software renderer it is.
    unsafe {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = MainWindowViewModel::new();

    SqualrEngine::initialize(true);

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(e) => {
            Logger::get_instance().log(LogLevel::Error, "Fatal error starting Squalr.", Some(e.to_string().as_str()));
        }
    }
}
