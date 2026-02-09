use crate::structures::processes::process_icon::ProcessIcon;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    process_id: u32,
    name: String,
    is_windowed: bool,
    icon: Option<ProcessIcon>,
}

impl ProcessInfo {
    pub fn new(
        process_id: u32,
        name: String,
        is_windowed: bool,
        icon: Option<ProcessIcon>,
    ) -> Self {
        Self {
            process_id,
            name,
            is_windowed,
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

    pub fn get_is_windowed(&self) -> bool {
        self.is_windowed
    }

    pub fn get_icon(&self) -> &Option<ProcessIcon> {
        &self.icon
    }

    pub fn set_icon(
        &mut self,
        icon: Option<ProcessIcon>,
    ) {
        self.icon = icon;
    }
}
