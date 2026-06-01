use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum PluginPermission {
    AttachDebugger,
    ControlDebuggerExecution,
    ManageDebuggerBreakpoints,
    ReadSymbolStore,
    WriteSymbolStore,
    ReadSymbolTreeWindow,
    WriteSymbolTreeWindow,
    ReadProcessMemory,
    ReadRegisters,
    WriteProcessMemory,
    WriteRegisters,
}

impl PluginPermission {
    pub fn get_display_name(&self) -> &'static str {
        match self {
            Self::AttachDebugger => "Attach debugger",
            Self::ControlDebuggerExecution => "Control debugger execution",
            Self::ManageDebuggerBreakpoints => "Manage debugger breakpoints",
            Self::ReadSymbolStore => "Read symbol store",
            Self::WriteSymbolStore => "Write symbol store",
            Self::ReadSymbolTreeWindow => "Read Symbol Tree window",
            Self::WriteSymbolTreeWindow => "Write Symbol Tree window",
            Self::ReadProcessMemory => "Read process memory",
            Self::ReadRegisters => "Read registers",
            Self::WriteProcessMemory => "Write process memory",
            Self::WriteRegisters => "Write registers",
        }
    }
}
