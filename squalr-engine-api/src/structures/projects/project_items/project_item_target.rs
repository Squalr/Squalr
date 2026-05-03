use serde::{Deserialize, Serialize};

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
}

impl ProjectItemTarget {
    pub fn new_address(
        address: u64,
        module_name: String,
    ) -> Self {
        Self::Address { address, module_name }
    }
}
