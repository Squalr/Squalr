use crate::{
    registries::data_types::data_type_registry::DataTypeRegistry,
    structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_ref::SymbolicStructRef, valued_struct::ValuedStruct},
};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicStructDefinition {
    symbol_namespace: String,
    fields: Vec<SymbolicFieldDefinition>,
}

impl SymbolicStructDefinition {
    pub fn new(
        symbol_namespace: String,
        fields: Vec<SymbolicFieldDefinition>,
    ) -> Self {
        SymbolicStructDefinition { symbol_namespace, fields }
    }

    pub fn new_anonymous(fields: Vec<SymbolicFieldDefinition>) -> Self {
        SymbolicStructDefinition {
            symbol_namespace: String::new(),
            fields,
        }
    }

    pub fn add_field(
        &mut self,
        symbolic_struct_field: SymbolicFieldDefinition,
    ) {
        self.fields.push(symbolic_struct_field);
    }

    pub fn get_valued_struct(
        &self,
        data_type_registry: &Arc<RwLock<DataTypeRegistry>>,
    ) -> ValuedStruct {
        let fields = self
            .fields
            .iter()
            .map(|field| field.get_valued_struct_field(data_type_registry, false))
            .collect();
        ValuedStruct::new(SymbolicStructRef::new(self.symbol_namespace.clone()), fields)
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

impl FromStr for SymbolicStructDefinition {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let JIRA = 696969;
        let fields: Result<Vec<SymbolicFieldDefinition>, Self::Err> = string
            .split(';')
            .filter(|&field_string| !field_string.is_empty())
            .map(|field_string| SymbolicFieldDefinition::from_str(field_string))
            .collect();

        Ok(SymbolicStructDefinition::new(String::new(), fields?))
    }
}
