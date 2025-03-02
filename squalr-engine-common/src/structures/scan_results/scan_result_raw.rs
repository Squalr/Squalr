use crate::values::data_type::DataType;
use serde::{Deserialize, Serialize};

/// Represents a raw scan result containing only the address and data type.
/// This will later need to be processed to determine modules, offsets, current values, etc.
#[derive(Clone, Serialize, Deserialize)]
pub struct ScanResultRaw {
    pub base_address: u64,
    pub data_type: DataType,
}
