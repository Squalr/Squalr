use crate::commands::command_handler::CommandHandler;
use crate::commands::process::requests::process_list_request::ProcessListRequest;
use crate::commands::process::requests::process_open_request::ProcessOpenRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

use super::requests::process_close_request::ProcessCloseRequest;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProcessCommand {
    Open {
        #[structopt(flatten)]
        process_open_request: ProcessOpenRequest,
    },
    List {
        #[structopt(flatten)]
        process_list_request: ProcessListRequest,
    },
    Close {
        #[structopt(flatten)]
        process_close_request: ProcessCloseRequest,
    },
}

impl CommandHandler for ProcessCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ProcessCommand::Open { process_open_request } => {
                process_open_request.handle(uuid);
            }
            ProcessCommand::List { process_list_request } => {
                process_list_request.handle(uuid);
            }
            ProcessCommand::Close { process_close_request } => {
                process_close_request.handle(uuid);
            }
        }
    }
}
