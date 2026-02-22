use crate::response_handlers::handle_engine_response;
use anyhow::{Result, anyhow};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::io;
use std::io::Write;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use structopt::clap::ErrorKind;

pub struct Cli {}

enum ParsedInput {
    Command(PrivilegedCommand),
    DisplayedHelpOrVersion,
}

/// Implements a command line listener polls for text input commands to control the engine.
impl Cli {
    pub fn run_loop(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            if let Err(error) = stdout.flush() {
                log::error!("Error flushing stdout {}", error);
                break;
            }

            let mut input = String::new();
            if let Err(error) = stdin.read_line(&mut input) {
                log::error!("Error reading input {}", error);
                break;
            }

            if !Self::handle_input(engine_unprivileged_state, input.trim()) {
                break;
            }
        }
    }

    pub fn stay_alive() {
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }

    /// Executes a single command and blocks until the engine response arrives.
    pub fn run_one_shot(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        raw_command_text: &str,
    ) -> Result<()> {
        let engine_command = match Self::parse_input(raw_command_text)? {
            ParsedInput::Command(engine_command) => engine_command,
            ParsedInput::DisplayedHelpOrVersion => return Ok(()),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);

        engine_unprivileged_state.dispatch_command(engine_command, move |engine_response| {
            handle_engine_response(engine_response);
            let _ = response_sender.send(());
        });

        response_receiver
            .recv()
            .map_err(|error| anyhow!("Failed waiting for one-shot command response: {}", error))?;

        Ok(())
    }

    fn handle_input(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        input: &str,
    ) -> bool {
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("close") || input.eq_ignore_ascii_case("quit") {
            return false;
        }

        let engine_command = match Self::parse_input(input) {
            Ok(ParsedInput::Command(engine_command)) => engine_command,
            Ok(ParsedInput::DisplayedHelpOrVersion) => return true,
            Err(error) => {
                log::error!("Error parsing engine command: {}", error);
                return true;
            }
        };

        engine_unprivileged_state.dispatch_command(engine_command, |engine_command| {
            handle_engine_response(engine_command);
        });

        true
    }

    fn parse_input(input: &str) -> Result<ParsedInput> {
        let mut cli_command = shlex::split(input).ok_or_else(|| anyhow!("Error parsing input"))?;

        if cli_command.is_empty() {
            return Err(anyhow!("No command provided"));
        }

        // Inject a synthetic binary name so command text can be parsed as a CLI argv list.
        cli_command.insert(0, String::from("squalr-cli"));

        match PrivilegedCommand::from_iter_safe(&cli_command) {
            Ok(engine_command) => Ok(ParsedInput::Command(engine_command)),
            Err(error) => {
                if matches!(error.kind, ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed) {
                    print!("{}", error);
                    Ok(ParsedInput::DisplayedHelpOrVersion)
                } else {
                    Err(anyhow!(error.to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, ParsedInput};

    #[test]
    fn parse_input_returns_help_for_top_level_help_flag() {
        let parsed_input = Cli::parse_input("--help").expect("Expected --help to be handled as a display-only command");

        assert!(matches!(parsed_input, ParsedInput::DisplayedHelpOrVersion));
    }

    #[test]
    fn parse_input_returns_command_for_valid_process_list_command() {
        let parsed_input = Cli::parse_input("process list").expect("Expected process list command to parse successfully");

        assert!(matches!(parsed_input, ParsedInput::Command(_)));
    }
}
