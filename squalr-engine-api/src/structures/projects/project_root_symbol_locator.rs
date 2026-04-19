use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectRootSymbolLocator {
    AbsoluteAddress { address: u64 },
    ModuleOffset { module_name: String, offset: u64 },
}

impl ProjectRootSymbolLocator {
    pub fn new_absolute_address(address: u64) -> Self {
        Self::AbsoluteAddress { address }
    }

    pub fn new_module_offset(
        module_name: String,
        offset: u64,
    ) -> Self {
        Self::ModuleOffset { module_name, offset }
    }

    pub fn get_focus_address(&self) -> u64 {
        match self {
            Self::AbsoluteAddress { address } => *address,
            Self::ModuleOffset { offset, .. } => *offset,
        }
    }

    pub fn get_focus_module_name(&self) -> &str {
        match self {
            Self::AbsoluteAddress { .. } => "",
            Self::ModuleOffset { module_name, .. } => module_name,
        }
    }
}

impl fmt::Display for ProjectRootSymbolLocator {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::AbsoluteAddress { address } => write!(formatter, "0x{:X}", address),
            Self::ModuleOffset { module_name, offset } => write!(formatter, "{} + 0x{:X}", module_name, offset),
        }
    }
}
