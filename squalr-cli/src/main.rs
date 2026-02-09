mod cli;
mod response_handlers;

use anyhow::{Context, Result, bail};
use cli::Cli;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;

fn main() -> Result<()> {
    let command_line_arguments: Vec<String> = std::env::args().collect();
    let engine_mode = if command_line_arguments
        .iter()
        .any(|argument| argument == "--ipc-mode")
    {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };
    let one_shot_command_text = build_one_shot_command_text(&command_line_arguments);

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
        if let Some(one_shot_command_text) = one_shot_command_text {
            Cli::run_one_shot(engine_unprivileged_state, &one_shot_command_text).context("Failed running one-shot CLI command.")?;
        } else {
            Cli::run_loop(engine_unprivileged_state);
        }
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("CLI running as a privileged IPC shell.");

        // Keep the CLI alive, exiting on any user input. Generally this is an invisible process, so it's just a way to keep the app running.
        Cli::stay_alive();
    } else {
        bail!("Unsupported CLI state.");
    }

    Ok(())
}

fn build_one_shot_command_text(command_line_arguments: &[String]) -> Option<String> {
    let one_shot_tokens: Vec<String> = command_line_arguments
        .iter()
        .skip(1)
        .filter(|argument| argument.as_str() != "--ipc-mode")
        .cloned()
        .collect();

    if one_shot_tokens.is_empty() { None } else { Some(one_shot_tokens.join(" ")) }
}
