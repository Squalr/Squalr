use crate::{
    registries::data_types::data_type_registry::DataTypeRegistry,
    structures::structs::{
        symbolic_struct_field_definition::SymbolicStructFieldDefinition, symbolic_struct_ref::SymbolicStructRef, valued_struct::ValuedStruct,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicStructDefinition {
    namespace: String,
    fields: Vec<SymbolicStructFieldDefinition>,
}

impl SymbolicStructDefinition {
    pub fn new(
        namespace: String,
        fields: Vec<SymbolicStructFieldDefinition>,
    ) -> Self {
        SymbolicStructDefinition { namespace, fields }
    }

    pub fn add_field(
        &mut self,
        symbolic_struct_field: SymbolicStructFieldDefinition,
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
        ValuedStruct::new(SymbolicStructRef::new(self.namespace.clone()), fields)
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
        let fields: Result<Vec<SymbolicStructFieldDefinition>, Self::Err> = string
            .split(';')
            .filter(|&field_string| !field_string.is_empty())
            .map(|field_string| SymbolicStructFieldDefinition::from_str(field_string))
            .collect();

        Ok(SymbolicStructDefinition::new(String::new(), fields?))
    }
}
