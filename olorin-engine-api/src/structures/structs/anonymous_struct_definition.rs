use crate::{
    registries::data_types::data_type_registry::DataTypeRegistry, structures::structs::symbolic_struct_field_definition::SymbolicStructFieldDefinition,
};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnonymousStructDefinition {
    fields: Vec<SymbolicStructFieldDefinition>,
}

impl AnonymousStructDefinition {
    pub fn new(fields: Vec<SymbolicStructFieldDefinition>) -> Self {
        AnonymousStructDefinition { fields }
    }

    pub fn add_field(
        &mut self,
        symbolic_struct_field: SymbolicStructFieldDefinition,
    ) {
        self.fields.push(symbolic_struct_field);
    }

    pub fn get_size_in_bytes(
        &self,
        data_type_registry: &Arc<RwLock<DataTypeRegistry>>,
    ) -> u64 {
        self.fields
            .iter()
            .map(|field| field.get_size_in_bytes(data_type_registry))
            .sum()
    }
}

impl FromStr for AnonymousStructDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let fields: Result<Vec<SymbolicStructFieldDefinition>, Self::Err> = string
            .split(';')
            .filter(|&field_string| !field_string.is_empty())
            .map(|field_string| SymbolicStructFieldDefinition::from_str(field_string))
            .collect();

        Ok(AnonymousStructDefinition::new(fields?))
    }
}
