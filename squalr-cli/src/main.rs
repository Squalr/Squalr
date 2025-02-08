mod cli;
mod logging;
mod response_handlers;

use crate::logging::cli_log_listener::CliLogListener;
use cli::Cli;
use squalr_engine::squalr_engine::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let engine_mode = if args.contains(&"--ipc-mode".to_string()) {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };

    // Hook into engine logging for the cli to display.
    let cli_log_listener = CliLogListener::new();
    Logger::get_instance().subscribe(cli_log_listener);

    // Start Squalr engine.
    SqualrEngine::initialize(engine_mode);

    if engine_mode == EngineMode::PrivilegedShell {
        Logger::get_instance().log(LogLevel::Info, "Spawning CLI as a privileged IPC shell.", None);
    }

    // Listen for user input.
    Cli::run_loop();
}
