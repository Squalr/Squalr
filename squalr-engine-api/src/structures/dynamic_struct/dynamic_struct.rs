use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicStruct {
    fields: Vec<DataTypeRef>,
}

impl DynamicStruct {
    pub fn new() -> Self {
        DynamicStruct { fields: vec![] }
    }

    pub fn add_field(
        &mut self,
        struct_field: DataTypeRef,
    ) {
        self.fields.push(struct_field);
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.fields.iter().map(|field| field.get_size_in_bytes()).sum()
    }
}

impl FromStr for DynamicStruct {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut dynamic_struct = DynamicStruct::new();
        let fields: Vec<&str> = s.split(';').filter(|&f| !f.is_empty()).collect();

        for field in fields {
            let struct_field = DataTypeRef::from_str(&field)?;

            dynamic_struct.add_field(struct_field);
        }

        Ok(dynamic_struct)
    }
}
