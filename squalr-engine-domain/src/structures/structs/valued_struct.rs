use crate::{
    structures::structs::{
        symbol_resolver::SymbolResolver,
        symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
        symbolic_struct_ref::SymbolicStructRef,
        valued_struct_error::ValuedStructError,
        valued_struct_field::{ValuedStructField, ValuedStructFieldData},
    },
    traits::from_string_privileged::FromStringPrivileged,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValuedStruct {
    symbolic_struct_ref: SymbolicStructRef,
    #[serde(default, skip_serializing_if = "SymbolicLayoutKind::is_default")]
    layout_kind: SymbolicLayoutKind,
    fields: Vec<ValuedStructField>,
}

impl ValuedStruct {
    pub fn new(
        symbolic_struct_ref: SymbolicStructRef,
        fields: Vec<ValuedStructField>,
    ) -> Self {
        ValuedStruct {
            symbolic_struct_ref,
            layout_kind: SymbolicLayoutKind::Struct,
            fields,
        }
    }

    pub fn new_with_layout_kind(
        symbolic_struct_ref: SymbolicStructRef,
        layout_kind: SymbolicLayoutKind,
        fields: Vec<ValuedStructField>,
    ) -> Self {
        ValuedStruct {
            symbolic_struct_ref,
            layout_kind,
            fields,
        }
    }

    pub fn new_anonymous(fields: Vec<ValuedStructField>) -> Self {
        ValuedStruct {
            symbolic_struct_ref: SymbolicStructRef::new_anonymous(),
            layout_kind: SymbolicLayoutKind::Struct,
            fields,
        }
    }

    pub fn new_anonymous_with_layout_kind(
        layout_kind: SymbolicLayoutKind,
        fields: Vec<ValuedStructField>,
    ) -> Self {
        ValuedStruct {
            symbolic_struct_ref: SymbolicStructRef::new_anonymous(),
            layout_kind,
            fields,
        }
    }

    pub fn get_struct_layout(
        &self,
        symbol_resolver: &impl SymbolResolver,
    ) -> Result<SymbolicStructDefinition, ValuedStructError> {
        let symbolic_struct_namespace = self.symbolic_struct_ref.get_symbolic_struct_namespace().trim();

        if symbolic_struct_namespace.is_empty() {
            return Err(ValuedStructError::AnonymousStructReference);
        }

        symbol_resolver
            .get_struct_layout(symbolic_struct_namespace)
            .as_deref()
            .cloned()
            .ok_or_else(|| ValuedStructError::symbolic_struct_not_registered(symbolic_struct_namespace.to_string()))
    }

    pub fn get_symbolic_struct_ref(&self) -> &SymbolicStructRef {
        &self.symbolic_struct_ref
    }

    pub fn get_layout_kind(&self) -> SymbolicLayoutKind {
        self.layout_kind
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        if self.layout_kind.is_union() {
            self.fields
                .iter()
                .map(|field| field.get_size_in_bytes())
                .max()
                .unwrap_or(0)
        } else {
            self.fields.iter().map(|field| field.get_size_in_bytes()).sum()
        }
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

    pub fn remove_field(
        &mut self,
        field_name: &str,
    ) -> bool {
        let field_count_before_removal = self.fields.len();
        self.fields.retain(|field| field.get_name() != field_name);

        self.fields.len() != field_count_before_removal
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        if !self.layout_kind.is_union() {
            return self.fields.iter().flat_map(|field| field.get_bytes()).collect();
        }

        let mut bytes = vec![0_u8; self.get_size_in_bytes() as usize];
        for field in &self.fields {
            let field_bytes = field.get_bytes();
            let copy_length = bytes.len().min(field_bytes.len());

            bytes[..copy_length].copy_from_slice(&field_bytes[..copy_length]);
        }

        bytes
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) -> bool {
        let total_size = bytes.len() as u64;
        let expected_size = self.get_size_in_bytes();

        debug_assert!(total_size == expected_size);

        if total_size != expected_size {
            return false;
        }

        if self.layout_kind.is_union() {
            for field in self.fields.iter_mut() {
                let field_size = field.get_size_in_bytes() as usize;

                if field_size > bytes.len() {
                    return false;
                }

                field.copy_from_bytes(&bytes[..field_size]);
            }

            return true;
        }

        let mut accumulated_size = 0u64;
        for field in self.fields.iter_mut() {
            let field_size = field.get_size_in_bytes();

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
                    .any(|field_b| field_a.get_name() == field_b.get_name())
            })
        });

        ValuedStruct::new_anonymous_with_layout_kind(first_struct.layout_kind, std::mem::take(&mut first_struct.fields))
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

impl<ContextType> FromStringPrivileged<ContextType> for ValuedStruct {
    type Err = String;

    fn from_string_privileged(
        string: &str,
        context: &ContextType,
    ) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = string.splitn(2, ':').collect();
        let struct_ref = SymbolicStructRef::new(parts.get(0).unwrap_or(&"").to_string());

        let field_data = parts.get(1).unwrap_or(&"");
        let fields = field_data
            .split(';')
            .filter(|string| !string.trim().is_empty())
            .map(|string| ValuedStructField::from_string_privileged(string, context))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ValuedStruct::new(struct_ref, fields))
    }
}

#[cfg(test)]
mod tests {
    use super::ValuedStruct;
    use crate::structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, built_in_types::u32::data_type_u32::DataTypeU32},
        structs::symbolic_struct_definition::SymbolicLayoutKind,
    };

    #[test]
    fn combine_exclusive_keeps_common_field_names_when_values_differ() {
        let first_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU8::get_value_from_primitive(10).to_named_valued_struct_field("value".to_string(), false),
            DataTypeU8::get_value_from_primitive(20).to_named_valued_struct_field("other".to_string(), false),
        ]);
        let second_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU8::get_value_from_primitive(30).to_named_valued_struct_field("value".to_string(), false),
            DataTypeU8::get_value_from_primitive(40).to_named_valued_struct_field("other".to_string(), false),
        ]);

        let combined_struct = ValuedStruct::combine_exclusive(&[first_struct, second_struct]);

        assert!(combined_struct.get_field("value").is_some());
        assert!(combined_struct.get_field("other").is_some());
        assert_eq!(combined_struct.get_fields().len(), 2);
    }

    #[test]
    fn combine_exclusive_discards_fields_not_present_in_all_structs() {
        let first_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU8::get_value_from_primitive(10).to_named_valued_struct_field("value".to_string(), false),
            DataTypeU8::get_value_from_primitive(20).to_named_valued_struct_field("only_first".to_string(), false),
        ]);
        let second_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU8::get_value_from_primitive(30).to_named_valued_struct_field("value".to_string(), false),
        ]);

        let combined_struct = ValuedStruct::combine_exclusive(&[first_struct, second_struct]);

        assert!(combined_struct.get_field("value").is_some());
        assert!(combined_struct.get_field("only_first").is_none());
        assert_eq!(combined_struct.get_fields().len(), 1);
    }

    #[test]
    fn remove_field_drops_matching_field_name() {
        let mut valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU8::get_value_from_primitive(10).to_named_valued_struct_field("value".to_string(), false),
            DataTypeU8::get_value_from_primitive(20).to_named_valued_struct_field("other".to_string(), false),
        ]);

        let did_remove_field = valued_struct.remove_field("value");

        assert!(did_remove_field);
        assert!(valued_struct.get_field("value").is_none());
        assert!(valued_struct.get_field("other").is_some());
    }

    #[test]
    fn union_layout_copies_same_bytes_to_all_fields() {
        let mut valued_struct = ValuedStruct::new_anonymous_with_layout_kind(
            SymbolicLayoutKind::Union,
            vec![
                DataTypeU32::get_value_from_primitive(0).to_named_valued_struct_field("as_u32".to_string(), false),
                DataTypeU8::get_value_from_primitive(0).to_named_valued_struct_field("first_byte".to_string(), false),
            ],
        );

        assert_eq!(valued_struct.get_size_in_bytes(), 4);
        assert!(valued_struct.copy_from_bytes(&[0x78, 0x56, 0x34, 0x12]));
        assert_eq!(
            valued_struct
                .get_field("as_u32")
                .and_then(|field| field.get_data_value())
                .map(|data_value| data_value.get_value_bytes().clone()),
            Some(vec![0x78, 0x56, 0x34, 0x12])
        );
        assert_eq!(
            valued_struct
                .get_field("first_byte")
                .and_then(|field| field.get_data_value())
                .map(|data_value| data_value.get_value_bytes().clone()),
            Some(vec![0x78])
        );
    }
}
