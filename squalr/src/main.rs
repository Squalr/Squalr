mod cli_log_listener;

use cli_log_listener::CliLogListener;
use slint::ComponentHandle;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

slint::include_modules!();

fn main() {
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

    let main_window = MainWindow::new().unwrap();
    main_window.run().unwrap();
}
