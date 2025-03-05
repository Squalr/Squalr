use crate::structures::{data_types::data_type_ref::DataTypeRef, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

/// Represents a base scan result containing only the address and data type.
/// This will later need to be processed to determine modules, offsets, current values, etc.
#[derive(Clone, Serialize, Deserialize)]
pub struct ScanResultBase {
    pub address: u64,
    pub data_type: DataTypeRef,
    pub current_value: Option<DataValue>,
    pub previous_value: Option<DataValue>,
}
