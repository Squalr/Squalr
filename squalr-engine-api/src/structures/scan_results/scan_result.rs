use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scan_results::scan_result_valued::ScanResultValued;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Reflect, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    valued_result: ScanResultValued,
    module: String,
    module_offset: u64,
    recently_read_value: Option<DataValue>,
    is_frozen: bool,
}

impl ScanResult {
    pub fn new(
        valued_result: ScanResultValued,
        module: String,
        module_offset: u64,
        recently_read_value: Option<DataValue>,
        is_frozen: bool,
    ) -> Self {
        Self {
            valued_result,
            module,
            module_offset,
            recently_read_value,
            is_frozen,
        }
    }

    pub fn get_address(&self) -> u64 {
        self.valued_result.get_address()
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.valued_result.get_data_type()
    }

    pub fn is_module(&self) -> bool {
        !self.module.is_empty()
    }

    pub fn get_module(&self) -> &str {
        &self.module
    }

    pub fn get_module_offset(&self) -> u64 {
        self.module_offset
    }

    pub fn get_recently_read_value(&self) -> &Option<DataValue> {
        &self.recently_read_value
    }

    pub fn get_current_value(&self) -> &Option<DataValue> {
        &self.valued_result.get_current_value()
    }

    pub fn get_previous_value(&self) -> &Option<DataValue> {
        &self.valued_result.get_previous_value()
    }

    pub fn get_is_frozen(&self) -> bool {
        self.is_frozen
    }
}

impl fmt::Debug for ScanResult {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.module.is_empty() {
            write!(formatter, "ScanResult {{ address: 0x{:X} }}", self.get_address())
        } else {
            write!(
                formatter,
                "ScanResult {{ module: {} }}, {{ offset: 0x{:X} }}, ",
                self.module, self.module_offset
            )
        }
    }
}
