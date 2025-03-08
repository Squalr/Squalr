use crate::structures::processes::process_info::OpenedProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessChangedEvent {
    pub process_info: Option<OpenedProcessInfo>,
}
