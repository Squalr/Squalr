pub use structopt::clap;

mod command;
mod memory;
mod parse_error;
mod parser;
mod plugins;
mod pointer_scan;
mod process;
mod project;
mod project_items;
mod project_symbols;
mod prompt_formatter;
mod registry;
mod root_command;
mod scan;
mod scan_results;
mod settings;
mod struct_scan;
mod trackable_tasks;

pub use command::CommandLineCommand;
pub use parse_error::CommandLineParseError;
pub use parser::{
    parse_command_line, parse_command_line_args, parse_command_line_with_program_name, parse_privileged_command, parse_prompt_command_line,
    parse_unprivileged_command,
};
pub use prompt_formatter::format_prompt_command_error;

#[cfg(test)]
mod tests;
