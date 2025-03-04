use crate::structures::scan_results::scan_result_base::ScanResultBase;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScanResult {
    base_result: ScanResultBase,
    module: String,
    module_offset: u64,
    recently_read_value: Option<Box<dyn DataValue>>,
}

impl ScanResult {
    pub fn new(
        base_result: ScanResultBase,
        module: String,
        module_offset: u64,
        recently_read_value: Option<Box<dyn DataValue>>,
    ) -> Self {
        Self {
            base_result,
            module,
            module_offset,
            recently_read_value,
        }
    }

    pub fn get_data_type(&self) -> &Box<dyn DataType> {
        &self.base_result.data_type
    }

    pub fn get_address(&self) -> u64 {
        self.base_result.address
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

    pub fn get_recently_read_value(&self) -> &Option<Box<dyn DataValue>> {
        &self.recently_read_value
    }

    pub fn get_current_value(&self) -> &Option<Box<dyn DataValue>> {
        &self.base_result.previous_value
    }

    pub fn get_previous_value(&self) -> &Option<Box<dyn DataValue>> {
        &self.base_result.previous_value
    }
}

impl fmt::Debug for ScanResult {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.module.is_empty() {
            write!(formatter, "ScanResult {{ address: 0x{:X} }}", self.base_result.address)
        } else {
            write!(
                formatter,
                "ScanResult {{ module: {} }}, {{ offset: 0x{:X} }}, ",
                self.module, self.module_offset
            )
        }
    }
}
