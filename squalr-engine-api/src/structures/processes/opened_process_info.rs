use crate::structures::memory::bitness::Bitness;
use crate::structures::processes::process_icon::ProcessIcon;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenedProcessInfo {
    process_id: u32,
    name: String,
    handle: u64,
    bitness: Bitness,
    icon: Option<ProcessIcon>,
}

impl OpenedProcessInfo {
    pub fn new(
        process_id: u32,
        name: String,
        handle: u64,
        bitness: Bitness,
        icon: Option<ProcessIcon>,
    ) -> Self {
        Self {
            process_id,
            name,
            handle,
            bitness,
            icon,
        }
    }

    pub fn get_process_id(&self) -> u32 {
        self.process_id
    }

    pub fn get_process_id_raw(&self) -> u32 {
        self.process_id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_handle(&self) -> u64 {
        self.handle
    }

    pub fn get_bitness(&self) -> Bitness {
        self.bitness
    }

    pub fn get_icon(&self) -> &Option<ProcessIcon> {
        &self.icon
    }
}
