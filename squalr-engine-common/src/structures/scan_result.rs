use crate::values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScanResult {
    base_address: u64,
    module: String,
    current_value: DataValue,
    previous_value: DataValue,
}

impl ScanResult {
    pub fn new(
        base_address: u64,
        module: String,
        current_value: DataValue,
        previous_value: DataValue,
    ) -> Self {
        Self {
            module,
            base_address,
            current_value,
            previous_value,
        }
    }

    pub fn get_base_address(&self) -> u64 {
        self.base_address
    }

    pub fn get_module(&self) -> &str {
        &self.module
    }

    pub fn get_current_value(&self) -> &DataValue {
        &self.current_value
    }

    pub fn get_previous_value(&self) -> &DataValue {
        &self.previous_value
    }
}

impl fmt::Debug for ScanResult {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            formatter,
            "ScanResult {{ base address: 0x{:X} }}, {{ module: {} }}",
            self.base_address, self.module
        )
    }
}
