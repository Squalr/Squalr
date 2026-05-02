use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectSymbolLocator {
    AbsoluteAddress { address: u64 },
    ModuleOffset { module_name: String, offset: u64 },
}

impl ProjectSymbolLocator {
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

    pub fn to_locator_key(&self) -> String {
        match self {
            Self::AbsoluteAddress { address } => format!("absolute:{:X}", address),
            Self::ModuleOffset { module_name, offset } => format!("module:{}:{:X}", module_name, offset),
        }
    }

    pub fn rename_module(
        &mut self,
        old_module_name: &str,
        new_module_name: &str,
    ) {
        let Self::ModuleOffset { module_name, .. } = self else {
            return;
        };

        if module_name == old_module_name {
            *module_name = new_module_name.to_string();
        }
    }
}

impl fmt::Display for ProjectSymbolLocator {
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
