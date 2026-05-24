use crate::commands::process::close::process_close_request::ProcessCloseRequest;
use crate::commands::process::icon::process_icon_request::ProcessIconRequest;
use crate::commands::process::list::process_list_request::ProcessListRequest;
use crate::commands::process::open::process_open_request::ProcessOpenRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessCommand {
    Open { process_open_request: ProcessOpenRequest },
    List { process_list_request: ProcessListRequest },
    Icon { process_icon_request: ProcessIconRequest },
    Close { process_close_request: ProcessCloseRequest },
}
