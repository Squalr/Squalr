pub mod results_command_list;

use crate::command_handlers::results::results_command_list::handle_results_list;
use crate::commands::results::results_command::ResultsCommand;

pub fn handle_results_command(cmd: ResultsCommand) {
    match cmd {
        ResultsCommand::List { .. } => handle_results_list(cmd),
    }
}
