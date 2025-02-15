use crate::commands::process::responses::process_close_response::ProcessCloseResponse;
use crate::commands::process::responses::process_list_response::ProcessListResponse;
use crate::commands::process::responses::process_open_response::ProcessOpenResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessResponse {
    List { process_list_response: ProcessListResponse },
    Close { process_close_response: ProcessCloseResponse },
    Open { process_open_response: ProcessOpenResponse },
}
