use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryFreezeTarget {
    pub address: u64,
    pub module_name: String,
    pub data_type_id: String,
}
