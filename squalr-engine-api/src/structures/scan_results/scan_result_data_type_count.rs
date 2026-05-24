use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Serialize};

/// Stores the number of surviving scan results for a specific data type.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResultDataTypeCount {
    pub data_type_ref: DataTypeRef,
    pub result_count: u64,
}

impl ScanResultDataTypeCount {
    pub fn new(
        data_type_ref: DataTypeRef,
        result_count: u64,
    ) -> Self {
        Self { data_type_ref, result_count }
    }
}
