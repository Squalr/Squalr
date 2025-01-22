use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let _main_window_view = MainWindowViewModel::new();

    SqualrEngine::initialize(true);

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(err) => {
            Logger::get_instance().log(LogLevel::Error, "Fatal error starting Squalr.", Some(err.to_string().as_str()));
        }
    }
}
