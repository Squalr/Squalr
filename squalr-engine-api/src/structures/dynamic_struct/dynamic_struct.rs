use crate::structures::dynamic_struct::dynamic_struct_field::DynamicStructField;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicStruct {
    fields: Vec<DynamicStructField>,
}

impl DynamicStruct {
    pub fn new(fields: Vec<DynamicStructField>) -> Self {
        DynamicStruct { fields }
    }

    pub fn add_field(
        &mut self,
        dynamic_struct_field: DynamicStructField,
    ) {
        self.fields.push(dynamic_struct_field);
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.fields.iter().map(|field| field.get_size_in_bytes()).sum()
    }
}

impl FromStr for DynamicStruct {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let fields: Result<Vec<DynamicStructField>, Self::Err> = string
            .split(';')
            .filter(|&field_string| !field_string.is_empty())
            .map(|field_string| DynamicStructField::from_str(field_string))
            .collect();

        Ok(DynamicStruct::new(fields?))
    }
}
