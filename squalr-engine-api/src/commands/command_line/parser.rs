use super::command::CommandLineCommand;
use super::parse_error::CommandLineParseError;
use super::root_command::CommandLineRootCommand;
use crate::commands::command_line::clap;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use std::ffi::OsString;
use structopt::StructOpt;
pub fn parse_command_line(input: &str) -> Result<CommandLineCommand, CommandLineParseError> {
    parse_command_line_with_program_name(input, "squalr")
}

pub fn parse_prompt_command_line(input: &str) -> Result<CommandLineCommand, CommandLineParseError> {
    parse_command_line_with_program_name(input, "")
}

pub fn parse_command_line_with_program_name(
    input: &str,
    program_name: &str,
) -> Result<CommandLineCommand, CommandLineParseError> {
    let mut command_arguments = shlex::split(input).ok_or(CommandLineParseError::InvalidShellWords)?;

    if command_arguments.is_empty() {
        return Err(CommandLineParseError::EmptyCommand);
    }

    command_arguments.insert(0, program_name.to_string());

    parse_command_line_args(command_arguments).map_err(CommandLineParseError::Command)
}

pub fn parse_command_line_args<I, T>(iterator: I) -> Result<CommandLineCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    CommandLineRootCommand::from_iter_safe(iterator).map(Into::into)
}

pub fn parse_privileged_command<I, T>(iterator: I) -> Result<PrivilegedCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match parse_command_line_args(iterator)? {
        CommandLineCommand::Privileged(command) => Ok(command),
        CommandLineCommand::Unprivileged(_) => Err(clap::Error::with_description(
            "Expected a privileged command.",
            clap::ErrorKind::InvalidSubcommand,
        )),
    }
}

pub fn parse_unprivileged_command<I, T>(iterator: I) -> Result<UnprivilegedCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match parse_command_line_args(iterator)? {
        CommandLineCommand::Privileged(_) => Err(clap::Error::with_description(
            "Expected an unprivileged command.",
            clap::ErrorKind::InvalidSubcommand,
        )),
        CommandLineCommand::Unprivileged(command) => Ok(command),
    }
}
