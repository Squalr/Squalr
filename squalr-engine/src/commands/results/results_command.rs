use crate::commands::command_handler::CommandHandler;
use crate::commands::results::handlers::results_command_list::handle_results_list;
use serde::{Deserialize, Serialize};
use squalr_engine_common::values::data_type::DataType;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ResultsCommand {
    /// Collect values and scan in the same parallel thread pool.
    List {
        #[structopt(short = "p", long)]
        page: u64,

        #[structopt(short = "d", long)]
        data_type: DataType,
    },
}

impl CommandHandler for ResultsCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ResultsCommand::List { page, data_type } => {
                handle_results_list(*page, data_type, uuid);
            }
        }
    }
}
