use serde::{Deserialize, Serialize};
use squalr_engine_processes::process_info::{OpenedProcessInfo, ProcessInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessResponse {
    List { processes: Vec<ProcessInfo> },
    Close { process_info: OpenedProcessInfo },
    Open { process_info: ProcessInfo },
}
