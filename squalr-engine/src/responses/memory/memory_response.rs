use serde::{Deserialize, Serialize};
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemoryResponse {
    Read { value: DynamicStruct, address: u64, success: bool },
    Write { success: bool },
}
