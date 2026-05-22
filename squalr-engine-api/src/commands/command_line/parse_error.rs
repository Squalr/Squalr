use crate::commands::command_line::clap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandLineParseError {
    #[error("Error parsing input")]
    InvalidShellWords,
    #[error("No command provided")]
    EmptyCommand,
    #[error("{0}")]
    Command(#[from] clap::Error),
}
