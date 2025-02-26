mod cli;
mod logging;
mod response_handlers;

use crate::logging::cli_log_listener::CliLogListener;
use cli::Cli;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let engine_mode = if args.contains(&"--ipc-mode".to_string()) {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };

    // Hook into engine logging for the cli to display.
    let cli_log_listener = CliLogListener::new();
    Logger::subscribe(cli_log_listener);

    // Start Squalr engine.
    let squalr_engine = SqualrEngine::new(engine_mode);

    if engine_mode == EngineMode::PrivilegedShell {
        Logger::log(LogLevel::Info, "CLI running as a privileged IPC shell.", None);
    }

    // Listen for user input.
    Cli::run_loop();
}
