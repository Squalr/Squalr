mod cli;
mod logging;
mod response_handlers;

use crate::logging::cli_log_listener::CliLogListener;
use cli::Cli;
use olorin_engine::engine_mode::EngineMode;
use olorin_engine::olorin_engine::OlorinEngine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let engine_mode = if args.contains(&"--ipc-mode".to_string()) {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };

    // Start Olorin engine.
    let mut olorin_engine = match OlorinEngine::new(EngineMode::Standalone) {
        Ok(olorin_engine) => olorin_engine,
        Err(error) => panic!("Fatal error initializing Olorin engine: {}", error),
    };

    // Hook into engine logging for the cli to display.
    let _cli_log_listener = CliLogListener::new(
        match olorin_engine
            .get_engine_execution_context()
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
    olorin_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        let engine_execution_context = olorin_engine.get_engine_execution_context().as_ref().unwrap();

        // Listen for user input.
        // Note that the "Cli", when listening for input, is considered unprivileged, as it is considered the "UI".
        // Internally, these commands then get dispatched to an abstracted away privileged component.
        Cli::run_loop(engine_execution_context);
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("CLI running as a privileged IPC shell.");

        // Keep the CLI alive, exiting on any user input. Generally this is an invisible process, so it's just a way to keep the app running.
        Cli::stay_alive();
    } else {
        unreachable!("Unsupported CLI state.")
    }
}
