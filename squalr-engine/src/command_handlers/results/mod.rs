pub mod results_command;
pub mod results_command_list;

use crate::command_handlers::results::results_command::ResultsCommand;
pub use results_command_list::handle_results_list;

pub fn handle_results_command(cmd: &mut ResultsCommand) {
    match cmd {
        ResultsCommand::List { .. } => handle_results_list(cmd),
    }
}
