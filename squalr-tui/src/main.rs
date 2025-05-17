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
        Err(err) => panic!("Fatal error initializing Squalr engine: {}", err),
    };

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    squalr_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        let engine_execution_context = squalr_engine.get_engine_execution_context().as_ref().unwrap();
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("TUI running as a privileged IPC shell.");
    } else {
        unreachable!("Unsupported TUI state.")
    }
}
