use crate::structures::{data_types::data_type_ref::DataTypeRef, scan_results::scan_result_ref::ScanResultRef};
use serde::{Deserialize, Serialize};

/// Represents a base scan result containing only the address and data type.
/// This will later need to be processed to determine modules, offsets, current values, etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultBase {
    address: u64,
    data_type_ref: DataTypeRef,
    scan_result_ref: ScanResultRef,
}

impl ScanResultBase {
    pub fn new(
        address: u64,
        data_type_ref: DataTypeRef,
        scan_result_ref: ScanResultRef,
    ) -> Self {
        Self {
            address,
            data_type_ref,
            scan_result_ref,
        }
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    pub fn get_scan_result_ref(&self) -> &ScanResultRef {
        &self.scan_result_ref
    }
}
