use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicStructField {
    data_type: DataTypeRef,
}

impl DynamicStructField {
    pub fn new(data_type: DataTypeRef) -> Self {
        DynamicStructField { data_type }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.data_type.get_size_in_bytes()
    }

    pub fn get_value(&self) -> &DataTypeRef {
        &self.data_type
    }
}

impl FromStr for DynamicStructField {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let data_type = DataTypeRef::from_str(string)?;

        Ok(DynamicStructField::new(data_type))
    }
}
