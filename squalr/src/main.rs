// Disable terminal from spawning. All relevant output is routed to the output view anyways.
#![windows_subsystem = "windows"]

use squalr_engine::app_provisioner::progress_tracker::ProgressTracker;
use squalr_engine::app_provisioner::updater::app_updater::AppUpdater;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

pub fn main() {
    // Override the default backend (femtovg => software). I wont want to rely on devs having QT installed,
    // and femtovg is shit at rendering font. Skia doesn't install properly on nightly, so software renderer it is.
    unsafe {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }

    let mut squalr_engine = match SqualrEngine::new(EngineMode::Standalone) {
        Ok(squalr_engine) => squalr_engine,
        Err(err) => panic!("Fatal error initializing Squalr engine: {}", err),
    };

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = match MainWindowViewModel::new(squalr_engine.get_dependency_container_mut()) {
        Ok(main_window_view) => main_window_view,
        Err(err) => panic!("Fatal error creating Squalr GUI: {}", err),
    };

    squalr_engine.initialize();

    AppUpdater::run_update(ProgressTracker::new());

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(err) => {
            panic!("Fatal error in Squalr event loop: {}", err);
        }
    }
}
