use super::*;
use crate as api;
use api::commands::debugger::debugger_command::DebuggerCommand;
use api::commands::patches::patches_command::PatchesCommand;
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
fn parse_command_line_args_routes_debugger_alias_to_privileged_namespace() {
    let parsed_command = parse_command_line_args([
        "squalr-cli",
        "dbg",
        "breakpoint-set",
        "--address",
        "0x1000",
        "--access",
        "rw",
    ])
    .expect("Expected debugger command to parse.");

    assert!(matches!(
        parsed_command,
        CommandLineCommand::Privileged(api::commands::privileged_command::PrivilegedCommand::Debugger(
            DebuggerCommand::BreakpointSet { .. }
        ))
    ));
}

#[test]
fn parse_command_line_routes_find_what_writes_to_trace_start() {
    let parsed_command = parse_command_line_args([
        "squalr-cli",
        "dbg",
        "find-what-writes",
        "--address",
        "0x1000",
        "--size-in-bytes",
        "4",
    ])
    .expect("Expected debugger find-what-writes command to parse.");

    let CommandLineCommand::Privileged(api::commands::privileged_command::PrivilegedCommand::Debugger(DebuggerCommand::TraceStart {
        debugger_trace_start_request,
    })) = parsed_command
    else {
        panic!("Expected find-what-writes to lower to debugger trace start.");
    };

    assert_eq!(debugger_trace_start_request.address, 0x1000);
    assert_eq!(debugger_trace_start_request.size_in_bytes, 4);
    assert_eq!(
        debugger_trace_start_request.access,
        api::structures::debugger::DebuggerDataBreakpointAccess::Write
    );
}

#[test]
fn parse_command_line_routes_patch_nop_to_privileged_namespace() {
    let parsed_command = parse_command_line_args([
        "squalr-cli",
        "patch",
        "nop",
        "--address",
        "0x1000",
        "--label",
        "skip",
    ])
    .expect("Expected patch nop command to parse.");

    let CommandLineCommand::Privileged(api::commands::privileged_command::PrivilegedCommand::Patches(PatchesCommand::NoOperation {
        patch_no_operation_request,
    })) = parsed_command
    else {
        panic!("Expected patch nop to lower to no-operation patch request.");
    };

    assert_eq!(patch_no_operation_request.address, 0x1000);
    assert_eq!(patch_no_operation_request.label.as_deref(), Some("skip"));
}

#[test]
fn parse_command_line_routes_patch_apply_bytes_to_privileged_namespace() {
    let parsed_command = parse_command_line_args([
        "squalr-cli",
        "patch",
        "apply",
        "--address",
        "0x1000",
        "--byte",
        "0x90",
        "--byte",
        "0xCC",
        "--kind",
        "generic",
    ])
    .expect("Expected patch apply command to parse.");

    let CommandLineCommand::Privileged(api::commands::privileged_command::PrivilegedCommand::Patches(PatchesCommand::Apply { patch_apply_request })) =
        parsed_command
    else {
        panic!("Expected patch apply to lower to patch apply request.");
    };

    assert_eq!(patch_apply_request.address, 0x1000);
    assert_eq!(patch_apply_request.patched_bytes, vec![0x90, 0xCC]);
    assert_eq!(patch_apply_request.kind, api::structures::patches::PatchKind::Generic);
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
fn prompt_command_top_level_error_uses_prompt_usage() {
    let parse_error = parse_prompt_command_line("24").expect_err("Expected parse failure.");
    let CommandLineParseError::Command(parse_error) = parse_error else {
        panic!("Expected clap parse error.");
    };

    let prompt_error_message = format_prompt_command_error(&parse_error);

    assert!(prompt_error_message.starts_with("error:"));
    assert!(prompt_error_message.contains("Usage: <COMMAND>"));
    assert!(!prompt_error_message.contains("squalr-engine-api"));
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
