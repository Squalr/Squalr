pub mod callback;
pub mod cli_log_listener;
pub mod mvc;
pub mod ui;

use crate::ui::main_window_view::MainWindowView;
use cli_log_listener::CliLogListener;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

slint::include_modules!();

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // Initialize cli log listener to route log output to command line
    let cli_log_listener = CliLogListener::new();

    let hardware_vector_size = vectors::get_hardware_vector_size();
    let hardware_vector_name = vectors::get_hardware_vector_name();

    Logger::get_instance().subscribe(cli_log_listener);
    Logger::get_instance().log(LogLevel::Info, "Logger initialized", None);
    Logger::get_instance().log(
        LogLevel::Info,
        format!(
            "CPU vector size for accelerated scans: {:?} bytes ({:?} bits), architecture: {}",
            hardware_vector_size,
            hardware_vector_size * 8,
            hardware_vector_name,
        )
        .as_str(),
        None,
    );

    let main_window = MainWindowView::new();

    main_window.run();
}
