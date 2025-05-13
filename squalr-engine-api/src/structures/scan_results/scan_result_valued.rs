use crate::structures::scan_results::scan_result_base::ScanResultBase;
use crate::structures::{data_types::data_type_ref::DataTypeRef, data_values::data_value::DataValue};
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a base scan result containing only the address and data type.
/// This will later need to be processed to determine modules, offsets, current values, etc.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
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
    ) -> Self {
        Self {
            scan_result_base: ScanResultBase::new(address, data_type_ref),
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

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.scan_result_base.get_data_type()
    }

    pub fn get_current_value(&self) -> &Option<DataValue> {
        &self.current_value
    }

    pub fn get_previous_value(&self) -> &Option<DataValue> {
        &self.previous_value
    }
}

impl FromStr for ScanResultValued {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.split(',').collect();
        if parts.len() < 2 {
            return Err("Input string must contain at least an address and data type".to_string());
        }

        let address = match parts[0].trim().parse::<u64>() {
            Ok(address) => address,
            Err(err) => {
                return Err(format!("Failed to parse address: {}", err));
            }
        };

        let data_type = parts[1].trim().parse::<DataTypeRef>()?;

        let current_value = if parts.len() > 2 && !parts[2].trim().is_empty() {
            Some(parts[2].trim().parse::<DataValue>()?)
        } else {
            None
        };

        let previous_value = if parts.len() > 3 && !parts[3].trim().is_empty() {
            Some(parts[3].trim().parse::<DataValue>()?)
        } else {
            None
        };

        let scan_result_base = ScanResultBase::new(address, data_type);

        Ok(ScanResultValued {
            scan_result_base,
            current_value,
            previous_value,
        })
    }
}
