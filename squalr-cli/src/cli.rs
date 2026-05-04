use crate::response_handlers::{handle_privileged_engine_response, handle_unprivileged_engine_response};
use anyhow::{Result, anyhow};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::io;
use std::io::Write;
use std::sync::{Arc, mpsc};
use structopt::StructOpt;
use structopt::clap::ErrorKind;

pub struct Cli {}

enum ParsedInput {
    PrivilegedCommand(PrivilegedCommand),
    UnprivilegedCommand(UnprivilegedCommand),
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
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        if let Err(error) = stdout.flush() {
            log::error!("Error flushing stdout {}", error);
            return;
        }

        let mut input = String::new();
        let _ = stdin.read_line(&mut input);
        log::error!("Exiting cli.");
    }

    /// Executes a single command and blocks until the engine response arrives.
    pub fn run_one_shot(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        raw_command_text: &str,
    ) -> Result<()> {
        let parsed_input = match Self::parse_input(raw_command_text)? {
            ParsedInput::PrivilegedCommand(engine_command) => ParsedInput::PrivilegedCommand(engine_command),
            ParsedInput::UnprivilegedCommand(engine_command) => ParsedInput::UnprivilegedCommand(engine_command),
            ParsedInput::DisplayedHelpOrVersion => return Ok(()),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);

        match parsed_input {
            ParsedInput::PrivilegedCommand(engine_command) => {
                engine_unprivileged_state.dispatch_command(engine_command, move |engine_response| {
                    handle_privileged_engine_response(engine_response);
                    let _ = response_sender.send(());
                });
            }
            ParsedInput::UnprivilegedCommand(engine_command) => {
                engine_unprivileged_state.dispatch_unprivileged_command(engine_command, move |engine_response| {
                    handle_unprivileged_engine_response(engine_response);
                    let _ = response_sender.send(());
                });
            }
            ParsedInput::DisplayedHelpOrVersion => {}
        }

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

        let parsed_input = match Self::parse_input(input) {
            Ok(ParsedInput::PrivilegedCommand(engine_command)) => ParsedInput::PrivilegedCommand(engine_command),
            Ok(ParsedInput::UnprivilegedCommand(engine_command)) => ParsedInput::UnprivilegedCommand(engine_command),
            Ok(ParsedInput::DisplayedHelpOrVersion) => return true,
            Err(error) => {
                log::error!("Error parsing engine command: {}", error);
                return true;
            }
        };

        match parsed_input {
            ParsedInput::PrivilegedCommand(engine_command) => {
                engine_unprivileged_state.dispatch_command(engine_command, |engine_response| {
                    handle_privileged_engine_response(engine_response);
                });
            }
            ParsedInput::UnprivilegedCommand(engine_command) => {
                engine_unprivileged_state.dispatch_unprivileged_command(engine_command, |engine_response| {
                    handle_unprivileged_engine_response(engine_response);
                });
            }
            ParsedInput::DisplayedHelpOrVersion => {}
        }

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
            Ok(engine_command) => return Ok(ParsedInput::PrivilegedCommand(engine_command)),
            Err(error) if matches!(error.kind, ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed) => {
                print!("{}", error);
                return Ok(ParsedInput::DisplayedHelpOrVersion);
            }
            Err(_error) => {}
        }

        match UnprivilegedCommand::from_iter_safe(&cli_command) {
            Ok(engine_command) => Ok(ParsedInput::UnprivilegedCommand(engine_command)),
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
    use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;

    #[test]
    fn parse_input_returns_help_for_top_level_help_flag() {
        let parsed_input = Cli::parse_input("--help").expect("Expected --help to be handled as a display-only command");

        assert!(matches!(parsed_input, ParsedInput::DisplayedHelpOrVersion));
    }

    #[test]
    fn parse_input_returns_command_for_valid_process_list_command() {
        let parsed_input = Cli::parse_input("process list").expect("Expected process list command to parse successfully");

        assert!(matches!(parsed_input, ParsedInput::PrivilegedCommand(_)));
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_symbols_list_command() {
        let parsed_input = Cli::parse_input("project_symbols list").expect("Expected project_symbols list command to parse successfully");

        assert!(matches!(parsed_input, ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectSymbols(_))));
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_list_command() {
        let parsed_input = Cli::parse_input("project list").expect("Expected project list command to parse successfully");

        assert!(matches!(parsed_input, ParsedInput::UnprivilegedCommand(UnprivilegedCommand::Project(_))));
    }
}
