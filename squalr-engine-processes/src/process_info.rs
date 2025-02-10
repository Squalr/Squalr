use serde::{Deserialize, Serialize};
use sysinfo::Pid;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Bitness {
    Bit32,
    Bit64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessIcon {
    pub bytes_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

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
