use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value that may later be interpreted
/// as many data types, and supporting values passed via command line.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    pub value_str: String,
    pub is_value_hex: bool,
}

impl AnonymousValue {
    pub fn new(
        value: &str,
        is_value_hex: bool,
    ) -> Self {
        AnonymousValue {
            value_str: value.to_string(),
            is_value_hex,
        }
    }

    pub fn deanonymize_value(
        &self,
        data_type_id: &str,
    ) -> Result<DataValue, String> {
        let registry = DataTypeRegistry::get_instance().get_registry();

        match registry.get(data_type_id) {
            Some(data_type) => {
                let deanonymized_value = data_type.deanonymize_value(&self);

                match deanonymized_value {
                    Ok(value) => Ok(DataValue::new(data_type_id, value)),
                    Err(err) => Err(err.to_string()),
                }
            }
            None => Err("Cannot deanonymize value: data type is not registered.".into()),
        }
    }
}

impl ToString for AnonymousValue {
    fn to_string(&self) -> String {
        self.value_str.clone()
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let is_value_hex = string.starts_with("0x");
        Ok(AnonymousValue::new(string, is_value_hex))
    }
}
