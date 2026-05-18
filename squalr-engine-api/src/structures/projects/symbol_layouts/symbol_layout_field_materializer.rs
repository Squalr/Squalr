use crate::structures::structs::{
    symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
    symbolic_struct_definition::SymbolicLayoutKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutPositionedField {
    offset_in_bytes: u64,
    size_in_bytes: u64,
    field_definition: SymbolicFieldDefinition,
}

impl SymbolLayoutPositionedField {
    pub fn new(
        offset_in_bytes: u64,
        size_in_bytes: u64,
        field_definition: SymbolicFieldDefinition,
    ) -> Self {
        Self {
            offset_in_bytes,
            size_in_bytes,
            field_definition,
        }
    }

    pub fn get_offset_in_bytes(&self) -> u64 {
        self.offset_in_bytes
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    pub fn get_field_definition(&self) -> &SymbolicFieldDefinition {
        &self.field_definition
    }
}

pub struct SymbolLayoutFieldMaterializer;

impl SymbolLayoutFieldMaterializer {
    pub fn materialize_positioned_fields(
        layout_kind: SymbolicLayoutKind,
        declared_size_in_bytes: Option<u64>,
        mut positioned_fields: Vec<SymbolLayoutPositionedField>,
    ) -> Result<Vec<SymbolicFieldDefinition>, String> {
        if layout_kind.is_union() {
            positioned_fields.sort_by(|left_field, right_field| {
                left_field
                    .offset_in_bytes
                    .cmp(&right_field.offset_in_bytes)
                    .then_with(|| {
                        left_field
                            .field_definition
                            .get_field_name()
                            .cmp(right_field.field_definition.get_field_name())
                    })
            });

            return Ok(positioned_fields
                .into_iter()
                .map(|positioned_field| positioned_field.field_definition)
                .collect());
        }

        positioned_fields.sort_by(|left_field, right_field| {
            left_field
                .offset_in_bytes
                .cmp(&right_field.offset_in_bytes)
                .then_with(|| {
                    left_field
                        .field_definition
                        .get_field_name()
                        .cmp(right_field.field_definition.get_field_name())
                })
        });

        let mut materialized_fields = Vec::new();
        let mut next_sequential_offset = 0_u64;

        for positioned_field in positioned_fields {
            if positioned_field.offset_in_bytes < next_sequential_offset {
                return Err(format!(
                    "Cannot place field `{}` at 0x{:X}; it overlaps an earlier layout field.",
                    positioned_field.field_definition.get_field_name(),
                    positioned_field.offset_in_bytes
                ));
            }

            if positioned_field.offset_in_bytes > next_sequential_offset {
                materialized_fields.push(SymbolicFieldDefinition::new_unassigned(
                    positioned_field
                        .offset_in_bytes
                        .saturating_sub(next_sequential_offset),
                ));
            }

            materialized_fields.push(
                positioned_field
                    .field_definition
                    .with_offset_resolution(SymbolicFieldOffsetResolution::Sequential),
            );
            next_sequential_offset = positioned_field
                .offset_in_bytes
                .saturating_add(positioned_field.size_in_bytes);
        }

        if let Some(declared_size_in_bytes) = declared_size_in_bytes
            && declared_size_in_bytes > next_sequential_offset
        {
            materialized_fields.push(SymbolicFieldDefinition::new_unassigned(
                declared_size_in_bytes.saturating_sub(next_sequential_offset),
            ));
        }

        Ok(materialized_fields)
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolLayoutFieldMaterializer, SymbolLayoutPositionedField};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicLayoutKind},
    };

    #[test]
    fn materializes_leading_and_tail_unassigned_fields() {
        let fields = SymbolLayoutFieldMaterializer::materialize_positioned_fields(
            SymbolicLayoutKind::Struct,
            Some(0x20),
            vec![SymbolLayoutPositionedField::new(
                0x10,
                4,
                SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
            )],
        )
        .expect("Expected materialized fields.");

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_unassigned_size_in_bytes(), Some(0x10));
        assert_eq!(fields[1].get_field_name(), "health");
        assert_eq!(fields[2].get_unassigned_size_in_bytes(), Some(0x0C));
    }
}
