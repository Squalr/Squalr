use super::*;
use crate as api;
use api::commands::process::process_command::ProcessCommand;
use api::commands::project::project_command::ProjectCommand;

#[test]
fn parse_command_line_args_routes_privileged_namespace_directly() {
    let parsed_command = parse_command_line_args(["squalr-cli", "process", "list"]).expect("Expected process list to parse.");

    assert!(matches!(
        parsed_command,
        CommandLineCommand::Privileged(api::commands::privileged_command::PrivilegedCommand::Process(ProcessCommand::List { .. }))
    ));
}

#[test]
fn parse_command_line_args_routes_unprivileged_namespace_directly() {
    let parsed_command = parse_command_line_args(["squalr-cli", "project", "list"]).expect("Expected project list to parse.");

    assert!(matches!(
        parsed_command,
        CommandLineCommand::Unprivileged(api::commands::unprivileged_command::UnprivilegedCommand::Project(ProjectCommand::List { .. }))
    ));
}

#[test]
fn parse_command_line_handles_shell_words_and_project_aliases() {
    let parsed_command = parse_command_line("p create --project-name 'quoted name'").expect("Expected project create alias to parse.");

    assert!(matches!(
        parsed_command,
        CommandLineCommand::Unprivileged(api::commands::unprivileged_command::UnprivilegedCommand::Project(ProjectCommand::Create { .. }))
    ));
}

#[test]
fn parse_command_line_with_program_name_uses_caller_program_name_in_help() {
    let parse_error = parse_command_line_with_program_name("process open unexpected", "squalr-gui").expect_err("Expected parse failure.");

    assert!(parse_error.to_string().contains("squalr-gui process open"));
}

#[test]
fn prompt_command_line_omits_program_name_from_usage() {
    let parse_error = parse_prompt_command_line("process open unexpected").expect_err("Expected parse failure.");
    let prompt_error_message = match parse_error {
        CommandLineParseError::Command(error) => format_prompt_command_error(&error),
        error => error.to_string(),
    };

    assert!(prompt_error_message.contains("process open"));
    assert!(!prompt_error_message.contains("squalr process open"));
    assert!(!prompt_error_message.contains("For more information try"));
}

#[test]
fn prompt_command_error_summary_keeps_usage_without_full_help_footer() {
    let parse_error = parse_prompt_command_line("process open unexpected").expect_err("Expected parse failure.");
    let CommandLineParseError::Command(parse_error) = parse_error else {
        panic!("Expected clap parse error.");
    };

    let prompt_error_message = format_prompt_command_error(&parse_error);

    assert!(prompt_error_message.starts_with("error:"));
    assert!(prompt_error_message.contains("Usage: process open"));
    assert!(!prompt_error_message.contains("USAGE:"));
    assert!(!prompt_error_message.contains("For more information try"));
}

#[test]
fn prompt_command_help_is_compact_for_terminal_output() {
    let parse_error = parse_prompt_command_line("scan new --help").expect_err("Expected help response.");
    let CommandLineParseError::Command(parse_error) = parse_error else {
        panic!("Expected clap help response.");
    };

    let prompt_help_message = format_prompt_command_error(&parse_error);

    assert_eq!(prompt_help_message, "Usage: scan new");
    assert!(!prompt_help_message.contains("scan-new"));
    assert!(!prompt_help_message.contains("--help"));
    assert!(!prompt_help_message.contains("--version"));
}

#[test]
fn specific_privileged_parser_rejects_unprivileged_commands() {
    let parse_error = parse_privileged_command(["squalr-cli", "project", "list"]).expect_err("Expected unprivileged command to be rejected.");

    assert!(matches!(parse_error.kind, clap::ErrorKind::InvalidSubcommand));
}

#[test]
fn specific_unprivileged_parser_rejects_privileged_commands() {
    let parse_error = parse_unprivileged_command(["squalr-cli", "process", "list"]).expect_err("Expected privileged command to be rejected.");

    assert!(matches!(parse_error.kind, clap::ErrorKind::InvalidSubcommand));
}
