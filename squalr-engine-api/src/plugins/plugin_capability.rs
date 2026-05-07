use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum PluginCapability {
    DataType,
    InstructionSet,
    MemoryView,
    SymbolTree,
}

impl PluginCapability {
    pub fn get_cli_label(&self) -> &'static str {
        match self {
            Self::DataType => "data-type",
            Self::InstructionSet => "instruction-set",
            Self::MemoryView => "memory-view",
            Self::SymbolTree => "symbol-tree",
        }
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            Self::DataType => "Data type",
            Self::InstructionSet => "Instruction set",
            Self::MemoryView => "Memory view",
            Self::SymbolTree => "Symbol Tree",
        }
    }
}
