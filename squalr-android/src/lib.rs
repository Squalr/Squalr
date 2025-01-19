use squalr_engine::session_manager::SessionManager;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    // Create and show the main window, which in turn will instantiate all dockable windows.
    let main_window_view = MainWindowViewModel::new();

    if let Ok(session_manager) = SessionManager::get_instance().read() {
        session_manager.initialize();
    } else {
        Logger::get_instance().log(LogLevel::Error, "Fatal error initializing session manager.", None);
    }

    // Run the slint window event loop until slint::quit_event_loop() is called.
    match slint::run_event_loop() {
        Ok(_) => {}
        Err(e) => {
            Logger::get_instance().log(LogLevel::Error, "Fatal error starting Squalr.", Some(e.to_string().as_str()));
        }
    }
}
