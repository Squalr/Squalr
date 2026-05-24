use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum PluginPermission {
    ReadSymbolStore,
    WriteSymbolStore,
    ReadSymbolTreeWindow,
    WriteSymbolTreeWindow,
    ReadProcessMemory,
    WriteProcessMemory,
}

impl PluginPermission {
    pub fn get_display_name(&self) -> &'static str {
        match self {
            Self::ReadSymbolStore => "Read symbol store",
            Self::WriteSymbolStore => "Write symbol store",
            Self::ReadSymbolTreeWindow => "Read Symbol Tree window",
            Self::WriteSymbolTreeWindow => "Write Symbol Tree window",
            Self::ReadProcessMemory => "Read process memory",
            Self::WriteProcessMemory => "Write process memory",
        }
    }
}
