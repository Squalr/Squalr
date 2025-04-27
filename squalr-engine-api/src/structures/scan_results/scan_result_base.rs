use crate::structures::data_types::data_type_ref::DataTypeRef;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a base scan result containing only the address and data type.
/// This will later need to be processed to determine modules, offsets, current values, etc.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultBase {
    address: u64,
    data_type_ref: DataTypeRef,
}

impl ScanResultBase {
    pub fn new(
        address: u64,
        data_type_ref: DataTypeRef,
    ) -> Self {
        Self { address, data_type_ref }
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.data_type_ref
    }
}

impl FromStr for ScanResultBase {
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

        let data_type_ref = parts[1].trim().parse::<DataTypeRef>()?;

        Ok(ScanResultBase { address, data_type_ref })
    }
}
