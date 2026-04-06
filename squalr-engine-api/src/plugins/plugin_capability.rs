use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum PluginCapability {
    DataType,
    MemoryView,
}

impl PluginCapability {
    pub fn get_cli_label(&self) -> &'static str {
        match self {
            Self::DataType => "data-type",
            Self::MemoryView => "memory-view",
        }
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            Self::DataType => "Data type",
            Self::MemoryView => "Memory view",
        }
    }
}
