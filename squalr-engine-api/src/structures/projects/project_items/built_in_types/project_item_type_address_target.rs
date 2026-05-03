use crate::structures::memory::pointer::Pointer;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectItemAddressTarget {
    Address {
        address: u64,
        #[serde(default)]
        module_name: String,
        #[serde(default)]
        pointer_offsets: Vec<i64>,
    },
    PointerPath {
        pointer: Pointer,
    },
}

impl ProjectItemAddressTarget {
    pub fn new_address(
        address: u64,
        module_name: String,
    ) -> Self {
        Self::Address {
            address,
            module_name,
            pointer_offsets: Vec::new(),
        }
    }

    pub fn new_address_with_pointer_offsets(
        address: u64,
        module_name: String,
        pointer_offsets: Vec<i64>,
    ) -> Self {
        Self::Address {
            address,
            module_name,
            pointer_offsets,
        }
    }

    pub fn new_pointer_path(pointer: Pointer) -> Self {
        Self::PointerPath { pointer }
    }
}
