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

    // Start Squalr engine.
    let squalr_engine = SqualrEngine::new(engine_mode);

    // Hook into engine logging for the cli to display.
    let _cli_log_listener = CliLogListener::new(match squalr_engine.get_logger().subscribe_to_logs() {
        Ok(listener) => listener,
        Err(err) => {
            panic!("Fatal error hooking into engine log events: {}", err);
        }
    });

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    squalr_engine.get_logger().start_log_event_sender();

    if engine_mode == EngineMode::PrivilegedShell {
        log::info!("CLI running as a privileged IPC shell.")
    }

    // Listen for user input.
    Cli::run_loop(squalr_engine.get_engine_execution_context());
}
