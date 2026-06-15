mod cli;
mod http_ipc_server;
mod response_handlers;

use anyhow::{Context, Result, bail};
use cli::Cli;
use http_ipc_server::HttpIpcServer;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_session::platform_log_hooks::initialize_platform_log_hooks_once;

fn main() -> Result<()> {
    initialize_platform_log_hooks_once("SqualrCli");

    let command_line_arguments: Vec<String> = std::env::args().collect();
    let engine_mode = if command_line_arguments
        .iter()
        .any(|argument| argument == "--ipc-mode")
    {
        EngineMode::PrivilegedShell
    } else {
        EngineMode::Standalone
    };
    let http_ipc_bind_address = parse_http_ipc_bind_address(&command_line_arguments)?;
    let one_shot_command_text = build_one_shot_command_text(&command_line_arguments);

    // Start Squalr engine.
    let mut squalr_engine = SqualrEngine::new(engine_mode).context("Fatal error initializing Squalr engine.")?;

    // Start the log event sending now that both the CLI and engine are ready to receive log messages.
    squalr_engine.initialize();

    if let Some(http_ipc_bind_address) = http_ipc_bind_address {
        let engine_privileged_state = squalr_engine
            .get_engine_privileged_state()
            .as_ref()
            .context("Engine privileged state was unavailable for HTTP IPC mode.")?;

        HttpIpcServer::new(http_ipc_bind_address, engine_privileged_state.clone()).run()?;
    } else if engine_mode == EngineMode::Standalone {
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
    let mut one_shot_tokens = Vec::new();
    let mut command_line_argument_index = 1;

    while command_line_argument_index < command_line_arguments.len() {
        let argument = &command_line_arguments[command_line_argument_index];

        if argument == "--ipc-mode" || argument.starts_with("--http-ipc=") {
            command_line_argument_index += 1;
            continue;
        }

        if argument == "--http-ipc" {
            command_line_argument_index += 2;
            continue;
        }

        one_shot_tokens.push(argument.clone());
        command_line_argument_index += 1;
    }

    if one_shot_tokens.is_empty() { None } else { Some(one_shot_tokens.join(" ")) }
}

fn parse_http_ipc_bind_address(command_line_arguments: &[String]) -> Result<Option<String>> {
    let mut command_line_argument_index = 1;

    while command_line_argument_index < command_line_arguments.len() {
        let argument = &command_line_arguments[command_line_argument_index];

        if let Some(bind_address) = argument.strip_prefix("--http-ipc=") {
            if bind_address.trim().is_empty() {
                bail!("--http-ipc requires a non-empty bind address.");
            }

            return Ok(Some(bind_address.to_string()));
        }

        if argument == "--http-ipc" {
            let Some(bind_address) = command_line_arguments.get(command_line_argument_index + 1) else {
                bail!("--http-ipc requires a bind address, for example --http-ipc 127.0.0.1:49321.");
            };

            if bind_address.trim().is_empty() {
                bail!("--http-ipc requires a non-empty bind address.");
            }

            return Ok(Some(bind_address.to_string()));
        }

        command_line_argument_index += 1;
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{build_one_shot_command_text, parse_http_ipc_bind_address};

    #[test]
    fn parses_http_ipc_bind_address_from_separate_argument() -> Result<()> {
        let arguments = vec![
            String::from("squalr-cli"),
            String::from("--ipc-mode"),
            String::from("--http-ipc"),
            String::from("127.0.0.1:49321"),
        ];

        assert_eq!(parse_http_ipc_bind_address(&arguments)?, Some(String::from("127.0.0.1:49321")));

        Ok(())
    }

    #[test]
    fn parses_http_ipc_bind_address_from_assignment_argument() -> Result<()> {
        let arguments = vec![
            String::from("squalr-cli"),
            String::from("--http-ipc=127.0.0.1:49321"),
        ];

        assert_eq!(parse_http_ipc_bind_address(&arguments)?, Some(String::from("127.0.0.1:49321")));

        Ok(())
    }

    #[test]
    fn rejects_http_ipc_without_bind_address() {
        let arguments = vec![String::from("squalr-cli"), String::from("--http-ipc")];

        assert!(parse_http_ipc_bind_address(&arguments).is_err());
    }

    #[test]
    fn removes_http_ipc_arguments_from_one_shot_command_text() {
        let arguments = vec![
            String::from("squalr-cli"),
            String::from("--http-ipc"),
            String::from("127.0.0.1:49321"),
            String::from("process"),
            String::from("list"),
        ];

        assert_eq!(build_one_shot_command_text(&arguments), Some(String::from("process list")));
    }
}
