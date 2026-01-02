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
    let mut squalr_engine = match SqualrEngine::new(EngineMode::Standalone) {
        Ok(squalr_engine) => squalr_engine,
        Err(error) => panic!("Fatal error initializing Squalr engine: {}", error),
    };

    // Hook into engine logging for the cli to display.
    let _cli_log_listener = CliLogListener::new(
        match squalr_engine
            .get_engine_unprivileged_state()
            .as_ref()
            .unwrap_or_else(|| panic!("Engine context failed to initialize."))
            .get_logger()
            .subscribe_to_logs()
        {
            Ok(listener) => listener,
            Err(error) => {
                panic!("Fatal error hooking into engine log events: {}", error);
            }
        },
    );

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    squalr_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        let engine_unprivileged_state = squalr_engine.get_engine_unprivileged_state().as_ref().unwrap();

        // Listen for user input.
        // Note that the "Cli", when listening for input, is considered unprivileged, as it is considered the "UI".
        // Internally, these commands then get dispatched to an abstracted away privileged component.
        Cli::run_loop(engine_unprivileged_state);
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("CLI running as a privileged IPC shell.");

        // Keep the CLI alive, exiting on any user input. Generally this is an invisible process, so it's just a way to keep the app running.
        Cli::stay_alive();
    } else {
        unreachable!("Unsupported CLI state.")
    }
}
