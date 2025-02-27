use crate::commands::process::close::process_close_request::ProcessCloseRequest;
use crate::commands::process::list::process_list_request::ProcessListRequest;
use crate::commands::process::open::process_open_request::ProcessOpenRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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
