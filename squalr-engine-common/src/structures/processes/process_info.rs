use crate::structures::bitness::Bitness;
use crate::structures::processes::process_icon::ProcessIcon;
use serde::{Deserialize, Serialize};
use sysinfo::Pid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub process_id: u32,
    pub name: String,
    pub is_windowed: bool,
    pub icon: Option<ProcessIcon>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenedProcessInfo {
    pub process_id: u32,
    pub name: String,
    pub handle: u64,
    pub bitness: Bitness,
    pub icon: Option<ProcessIcon>,
}

impl ProcessInfo {
    pub fn get_process_id(&self) -> Pid {
        Pid::from_u32(self.process_id)
    }
}

impl OpenedProcessInfo {
    pub fn get_process_id(&self) -> Pid {
        Pid::from_u32(self.process_id)
    }
}
