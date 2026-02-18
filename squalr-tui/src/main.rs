mod app;
mod state;
mod theme;
mod views;

use crate::app::{AppShell, TerminalGuard};
use anyhow::{Context, Result, bail};
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::{SqualrEngine, SqualrEngineOptions};
use std::time::Duration;

fn main() -> Result<()> {
    let command_line_arguments: Vec<String> = std::env::args().collect();
    let engine_mode = if command_line_arguments.contains(&"--ipc-mode".to_string()) {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };

    let mut squalr_engine = SqualrEngine::new_with_options(
        engine_mode,
        SqualrEngineOptions {
            enable_unprivileged_console_logging: false,
        },
    )
    .context("Fatal error initializing Squalr engine.")?;
    squalr_engine.initialize();

    if engine_mode == EngineMode::Standalone {
        squalr_engine
            .get_engine_unprivileged_state()
            .as_ref()
            .context("Engine unprivileged state was unavailable in standalone mode.")?;
    } else if engine_mode == EngineMode::UnprivilegedHost {
        log::info!("TUI running in unprivileged host mode.");
    } else if engine_mode == EngineMode::PrivilegedShell {
        log::info!("TUI running as a privileged IPC shell.");
    } else {
        bail!("Unsupported TUI state.");
    }

    let mut terminal_guard = TerminalGuard::new()?;
    let mut app_shell = AppShell::new(Duration::from_millis(100));
    app_shell.run(&mut terminal_guard, engine_mode, &mut squalr_engine)?;

    Ok(())
}
