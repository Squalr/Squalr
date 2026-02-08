mod cli;
mod logging;
mod response_handlers;

use anyhow::{Context, Result, bail};
use cli::Cli;
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
        let engine_unprivileged_state = squalr_engine
            .get_engine_unprivileged_state()
            .as_ref()
            .context("Engine unprivileged state was unavailable in standalone mode.")?;

        // Listen for user input.
        // Note that the "Cli", when listening for input, is considered unprivileged, as it is considered the "UI".
        // Internally, these commands then get dispatched to an abstracted away privileged component.
        Cli::run_loop(engine_unprivileged_state);
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("CLI running as a privileged IPC shell.");

        // Keep the CLI alive, exiting on any user input. Generally this is an invisible process, so it's just a way to keep the app running.
        Cli::stay_alive();
    } else {
        bail!("Unsupported CLI state.");
    }

    Ok(())
}
