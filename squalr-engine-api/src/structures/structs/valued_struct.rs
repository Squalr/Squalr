use crate::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use crate::structures::structs::valued_struct_field::ValuedStructField;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl FromStr for ValuedStruct {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = input.splitn(2, ':').collect();
        let struct_ref = SymbolicStructRef::new(parts.get(0).unwrap_or(&"").to_string());

        let field_data = parts.get(1).unwrap_or(&"");
        let fields = field_data
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(ValuedStructField::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ValuedStruct::new(struct_ref, fields))
    }
}
