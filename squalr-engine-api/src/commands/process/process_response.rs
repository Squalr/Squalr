use crate::commands::process::close::process_close_response::ProcessCloseResponse;
use crate::commands::process::list::process_list_response::ProcessListResponse;
use crate::commands::process::open::process_open_response::ProcessOpenResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessResponse {
    List { process_list_response: ProcessListResponse },
    Close { process_close_response: ProcessCloseResponse },
    Open { process_open_response: ProcessOpenResponse },
}
