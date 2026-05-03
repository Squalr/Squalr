use crate::structures::memory::pointer::Pointer;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectItemTarget {
    #[default]
    None,
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
    Plugin {
        target_type_id: String,
        #[serde(default)]
        payload: BTreeMap<String, String>,
    },
}

impl ProjectItemTarget {
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

    pub fn new_plugin(
        target_type_id: String,
        payload: BTreeMap<String, String>,
    ) -> Self {
        Self::Plugin { target_type_id, payload }
    }
}
