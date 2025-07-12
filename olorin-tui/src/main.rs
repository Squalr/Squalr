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

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    olorin_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        let engine_execution_context = olorin_engine.get_engine_execution_context().as_ref().unwrap();
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("TUI running as a privileged IPC shell.");
    } else {
        unreachable!("Unsupported TUI state.")
    }
}
