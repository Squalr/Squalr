use crate::registries::registries::Registries;
use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use crate::structures::structs::valued_struct_error::ValuedStructError;
use crate::structures::structs::valued_struct_field::ValuedStructField;
use crate::structures::structs::{symbolic_struct_ref::SymbolicStructRef, valued_struct_field::ValuedStructFieldData};
use crate::traits::from_string_privileged::FromStringPrivileged;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ValuedStruct {
    symbolic_struct_ref: SymbolicStructRef,
    fields: Vec<ValuedStructField>,
}

impl ValuedStruct {
    pub fn new(
        symbolic_struct_ref: SymbolicStructRef,
        fields: Vec<ValuedStructField>,
    ) -> Self {
        ValuedStruct { symbolic_struct_ref, fields }
    }

    pub fn new_anonymous(fields: Vec<ValuedStructField>) -> Self {
        ValuedStruct {
            symbolic_struct_ref: SymbolicStructRef::new_anonymous(),
            fields,
        }
    }

    pub fn get_symbolic_struct(
        &self,
        symbol_registry: &SymbolRegistry,
    ) -> Result<SymbolicStructDefinition, ValuedStructError> {
        let symbolic_struct_namespace = self.symbolic_struct_ref.get_symbolic_struct_namespace().trim();

        if symbolic_struct_namespace.is_empty() {
            return Err(ValuedStructError::AnonymousStructReference);
        }

        symbol_registry
            .get(symbolic_struct_namespace)
            .as_deref()
            .cloned()
            .ok_or_else(|| ValuedStructError::symbolic_struct_not_registered(symbolic_struct_namespace.to_string()))
    }

    pub fn get_symbolic_struct_ref(&self) -> &SymbolicStructRef {
        &self.symbolic_struct_ref
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.fields.iter().map(|field| field.get_size_in_bytes()).sum()
    }

    pub fn get_display_string(
        &self,
        pretty_print: bool,
    ) -> String {
        self.fields
            .iter()
            .map(|field| field.get_display_string(pretty_print, 0))
            .collect::<Vec<_>>()
            .join(if pretty_print { ",\n" } else { "," })
    }

    pub fn get_fields(&self) -> &[ValuedStructField] {
        &self.fields
    }

    pub fn get_fields_mut(&mut self) -> &mut [ValuedStructField] {
        &mut self.fields
    }

    pub fn get_field(
        &self,
        field_name: &str,
    ) -> Option<&ValuedStructField> {
        self.fields.iter().find(|field| field.get_name() == field_name)
    }

    pub fn get_field_mut(
        &mut self,
        field_name: &str,
    ) -> Option<&mut ValuedStructField> {
        self.fields
            .iter_mut()
            .find(|field| field.get_name() == field_name)
    }

    pub fn set_field_data(
        &mut self,
        field_name: &str,
        valued_field_data: ValuedStructFieldData,
        is_read_only: bool,
    ) {
        if let Some(field) = self
            .fields
            .iter_mut()
            .find(|field| field.get_name() == field_name)
        {
            field.set_field_data(valued_field_data);
        } else {
            self.fields
                .push(ValuedStructField::new(field_name.to_string(), valued_field_data, is_read_only));
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        self.fields.iter().flat_map(|field| field.get_bytes()).collect()
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) -> bool {
        let mut accumulated_size = 0u64;
        let total_size = bytes.len() as u64;
        let expected_size = self.get_size_in_bytes();

        debug_assert!(total_size == expected_size);

        if total_size != expected_size {
            return false;
        }

        for field in self.fields.iter_mut() {
            let field_size = field.get_size_in_bytes() as u64;

            if accumulated_size + field_size > total_size {
                return false;
            }

            field.copy_from_bytes(&bytes[accumulated_size as usize..(accumulated_size + field_size) as usize]);
            accumulated_size += field_size;
        }

        debug_assert!(accumulated_size == total_size);

        true
    }

    pub fn combine_exclusive(valued_structs: &[ValuedStruct]) -> ValuedStruct {
        let Some(mut first_struct) = valued_structs.first().cloned() else {
            return ValuedStruct::new_anonymous(vec![]);
        };

        first_struct.fields.retain(|field_a| {
            valued_structs.iter().skip(1).all(|other_valued_struct| {
                other_valued_struct
                    .fields
                    .iter()
                    .any(|field_b| field_a == field_b)
            })
        });

        ValuedStruct::new_anonymous(std::mem::take(&mut first_struct.fields))
    }
}

impl fmt::Display for ValuedStruct {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let field_strings = self
            .fields
            .iter()
            .map(|field| field.to_string())
            .collect::<Vec<_>>()
            .join(";");

        write!(formatter, "{}:{}", self.symbolic_struct_ref, field_strings)
    }
}

impl FromStringPrivileged for ValuedStruct {
    type Err = String;

    fn from_string_privileged(
        string: &str,
        registries: &Registries,
    ) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.splitn(2, ':').collect();
        let struct_ref = SymbolicStructRef::new(parts.get(0).unwrap_or(&"").to_string());

        let field_data = parts.get(1).unwrap_or(&"");
        let fields = field_data
            .split(';')
            .filter(|string| !string.trim().is_empty())
            .map(|string| ValuedStructField::from_string_privileged(string, registries))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ValuedStruct::new(struct_ref, fields))
    }
}
