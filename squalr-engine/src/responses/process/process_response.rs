use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::ProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessResponse {
    List { processes: Vec<ProcessInfo> },
}
