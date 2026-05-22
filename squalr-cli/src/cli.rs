use crate::response_handlers::{handle_privileged_engine_response, handle_unprivileged_engine_response};
use anyhow::{Result, anyhow};
use squalr_engine_api::commands::command_line::clap::ErrorKind;
use squalr_engine_api::commands::command_line::{CommandLineCommand, CommandLineParseError, parse_command_line_with_program_name};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::io;
use std::io::Write;
use std::sync::{Arc, mpsc};

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
        match parse_command_line_with_program_name(input, "squalr-cli") {
            Ok(CommandLineCommand::Privileged(engine_command)) => Ok(ParsedInput::PrivilegedCommand(engine_command)),
            Ok(CommandLineCommand::Unprivileged(engine_command)) => Ok(ParsedInput::UnprivilegedCommand(engine_command)),
            Err(CommandLineParseError::Command(error)) if matches!(error.kind, ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed) => {
                print!("{}", error);
                Ok(ParsedInput::DisplayedHelpOrVersion)
            }
            Err(error) => Err(anyhow!(error.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, ParsedInput};
    use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
    use squalr_engine_api::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
    use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;

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
    fn parse_input_returns_unprivileged_command_for_project_symbols_write_value_command() {
        let parsed_input = Cli::parse_input("project_symbols write-value --address 4660 --type u32 --field value -v '255;decimal;'")
            .expect("Expected project_symbols write-value command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::WriteValue {
            project_symbols_write_value_request,
        })) = parsed_input
        else {
            panic!("Expected project_symbols write-value command.");
        };

        assert_eq!(project_symbols_write_value_request.address, 4660);
        assert_eq!(project_symbols_write_value_request.symbol_type_id, "u32");
        assert_eq!(project_symbols_write_value_request.field_name, "value");
        assert_eq!(
            project_symbols_write_value_request
                .anonymous_value_string
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Decimal
        );
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_symbols_upsert_layout_command() {
        let parsed_input = Cli::parse_input("project_symbols upsert-layout --id player.stats --field health:u32 --field unassigned[4] --size 8")
            .expect("Expected project_symbols upsert-layout command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::UpsertLayout {
            project_symbols_upsert_layout_request,
        })) = parsed_input
        else {
            panic!("Expected project_symbols upsert-layout command.");
        };

        assert_eq!(project_symbols_upsert_layout_request.struct_layout_id, "player.stats");
        assert_eq!(project_symbols_upsert_layout_request.size_in_bytes, Some(8));
        assert_eq!(project_symbols_upsert_layout_request.field_definitions.len(), 2);
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_symbols_delete_layout_command() {
        let parsed_input =
            Cli::parse_input("project_symbols delete-layout --id player.stats").expect("Expected project_symbols delete-layout command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::DeleteLayout {
            project_symbols_delete_layout_request,
        })) = parsed_input
        else {
            panic!("Expected project_symbols delete-layout command.");
        };

        assert_eq!(project_symbols_delete_layout_request.struct_layout_id, "player.stats");
        assert_eq!(project_symbols_delete_layout_request.replacement_data_type_id, "u8");
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_symbols_upsert_resolver_command() {
        let parsed_input = Cli::parse_input(r#"project_symbols upsert-resolver --id inventory.count --definition-json "{\"root_node\":{\"Literal\":4}}""#)
            .expect("Expected project_symbols upsert-resolver command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::UpsertResolver {
            project_symbols_upsert_resolver_request,
        })) = parsed_input
        else {
            panic!("Expected project_symbols upsert-resolver command.");
        };

        assert_eq!(project_symbols_upsert_resolver_request.resolver_id, "inventory.count");
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_symbols_delete_resolver_command() {
        let parsed_input = Cli::parse_input("project_symbols delete-resolver --id inventory.count")
            .expect("Expected project_symbols delete-resolver command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::DeleteResolver {
            project_symbols_delete_resolver_request,
        })) = parsed_input
        else {
            panic!("Expected project_symbols delete-resolver command.");
        };

        assert_eq!(project_symbols_delete_resolver_request.resolver_id, "inventory.count");
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_items_write_value_command() {
        let parsed_input = Cli::parse_input("project_items write-value -p project_items/health.json --field value -v '255;decimal;'")
            .expect("Expected project_items write-value command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectItems(ProjectItemsCommand::WriteValue {
            project_items_write_value_request,
        })) = parsed_input
        else {
            panic!("Expected project_items write-value command.");
        };

        assert_eq!(
            project_items_write_value_request.project_item_path,
            std::path::PathBuf::from("project_items/health.json")
        );
        assert_eq!(project_items_write_value_request.field_name, "value");
        assert_eq!(
            project_items_write_value_request
                .anonymous_value_string
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Decimal
        );
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_items_strip_symbol_command() {
        let parsed_input = Cli::parse_input("project_items strip-symbol -p project_items/health.json")
            .expect("Expected project_items strip-symbol command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectItems(ProjectItemsCommand::StripSymbol {
            project_items_strip_symbol_request,
        })) = parsed_input
        else {
            panic!("Expected project_items strip-symbol command.");
        };

        assert_eq!(
            project_items_strip_symbol_request.project_item_paths,
            vec![std::path::PathBuf::from("project_items/health.json")]
        );
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_items_update_details_command() {
        let parsed_input = Cli::parse_input("project_items update-details -p project_items/health.json --property icon_id -v 'u64;string;'")
            .expect("Expected project_items update-details command to parse successfully");

        let ParsedInput::UnprivilegedCommand(UnprivilegedCommand::ProjectItems(ProjectItemsCommand::UpdateDetails {
            project_items_update_details_request,
        })) = parsed_input
        else {
            panic!("Expected project_items update-details command.");
        };

        assert_eq!(
            project_items_update_details_request.project_item_paths,
            vec![std::path::PathBuf::from("project_items/health.json")]
        );
        assert_eq!(project_items_update_details_request.property_name.as_deref(), Some("icon_id"));
        assert_eq!(
            project_items_update_details_request
                .anonymous_value_string
                .as_ref()
                .expect("Expected parsed value.")
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::String
        );
    }

    #[test]
    fn parse_input_returns_unprivileged_command_for_project_list_command() {
        let parsed_input = Cli::parse_input("project list").expect("Expected project list command to parse successfully");

        assert!(matches!(parsed_input, ParsedInput::UnprivilegedCommand(UnprivilegedCommand::Project(_))));
    }
}
