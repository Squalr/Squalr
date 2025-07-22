use crate::structures::scan_results::scan_result_base::ScanResultBase;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use crate::structures::{data_types::data_type_ref::DataTypeRef, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

/// Represents a base scan result containing only the address and data type.
/// This will later need to be processed to determine modules, offsets, current values, etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultValued {
    scan_result_base: ScanResultBase,
    current_value: Option<DataValue>,
    previous_value: Option<DataValue>,
}

impl ScanResultValued {
    pub fn new(
        address: u64,
        data_type_ref: DataTypeRef,
        current_value: Option<DataValue>,
        previous_value: Option<DataValue>,
        handle: ScanResultRef,
    ) -> Self {
        Self {
            scan_result_base: ScanResultBase::new(address, data_type_ref, handle),
            current_value,
            previous_value,
        }
    }

    pub fn get_base_result(&self) -> &ScanResultBase {
        &self.scan_result_base
    }

    pub fn get_address(&self) -> u64 {
        self.scan_result_base.get_address()
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.scan_result_base.get_data_type_ref()
    }

    pub fn get_current_value(&self) -> &Option<DataValue> {
        &self.current_value
    }

    pub fn get_previous_value(&self) -> &Option<DataValue> {
        &self.previous_value
    }
}
