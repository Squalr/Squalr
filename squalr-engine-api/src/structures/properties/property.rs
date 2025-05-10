use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
    name: String,
    value: DataValue,
}

impl Property {
    pub fn new(
        name: String,
        value: DataValue,
    ) -> Self {
        Self { name, value }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &DataValue {
        &self.value
    }
}
