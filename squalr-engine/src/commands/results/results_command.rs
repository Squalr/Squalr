use crate::commands::command_handler::CommandHandler;
use crate::commands::results::requests::results_list_request::ResultsListRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ResultsCommand {
    /// Collect values and scan in the same parallel thread pool.
    List {
        #[structopt(flatten)]
        results_list_request: ResultsListRequest,
    },
}

impl CommandHandler for ResultsCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ResultsCommand::List { results_list_request } => {
                results_list_request.handle(uuid);
            }
        }
    }
}
