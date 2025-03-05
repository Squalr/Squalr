use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a value as a string that can potentially be converted to an explicit type later.
/// This is particularly useful when scannining for a value that may later be interpreted
/// as many data types, and supporting values passed via command line.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AnonymousValue {
    value_str: String,
}

impl AnonymousValue {
    pub fn new(value: &str) -> Self {
        AnonymousValue { value_str: value.to_string() }
    }

    pub fn deanonymize_type(
        &self,
        target_type: &DataTypeRef,
    ) -> Result<Box<dyn DataValue>, String> {
        let value_and_type_str = format!("{}={}", self.value_str, target_type);

        match value_and_type_str.parse::<Box<dyn DataValue>>() {
            Ok(result) => Ok(result),
            Err(err) => Err(err.to_string()),
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
        Ok(AnonymousValue::new(string))
    }
}
