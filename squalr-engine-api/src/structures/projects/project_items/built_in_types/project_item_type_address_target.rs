use crate::structures::memory::pointer::Pointer;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectItemAddressTarget {
    Address {
        address: u64,
        #[serde(default)]
        module_name: String,
    },
    PointerPath {
        pointer: Pointer,
    },
    Symbol {
        symbol_locator_key: String,
    },
}

impl ProjectItemAddressTarget {
    pub fn new_address(
        address: u64,
        module_name: String,
    ) -> Self {
        Self::Address { address, module_name }
    }

    pub fn new_pointer_path(pointer: Pointer) -> Self {
        Self::PointerPath { pointer }
    }

    pub fn new_symbol(symbol_locator_key: String) -> Self {
        Self::Symbol { symbol_locator_key }
    }
}
