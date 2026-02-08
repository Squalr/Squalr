use anyhow::{Context, Result, bail};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let engine_mode = if args.contains(&"--ipc-mode".to_string()) {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };

    // Start Squalr engine.
    let mut squalr_engine = SqualrEngine::new(engine_mode).context("Fatal error initializing Squalr engine.")?;

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    squalr_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        squalr_engine
            .get_engine_unprivileged_state()
            .as_ref()
            .context("Engine unprivileged state was unavailable in standalone mode.")?;
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("TUI running as a privileged IPC shell.");
    } else {
        bail!("Unsupported TUI state.");
    }

    Ok(())
}
