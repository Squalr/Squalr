use squalr_engine_api::structures::{
    data_values::{container_type::ContainerType, data_value::DataValue},
    projects::project_symbol_locator::ProjectSymbolLocator,
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
};
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use std::collections::HashSet;

pub struct SymbolTreeScalarValue {}

impl SymbolTreeScalarValue {
    pub fn build_query(
        project_symbol_locator: &ProjectSymbolLocator,
        field_definition: &SymbolicFieldDefinition,
        field_size_in_bytes: u64,
    ) -> Option<VirtualSnapshotQuery> {
        if field_size_in_bytes == 0 || !Self::data_type_is_supported(field_definition.get_data_type_ref().get_data_type_id()) {
            return None;
        }

        let scalar_field_definition = SymbolicFieldDefinition::new_named(
            field_definition.get_field_name().to_string(),
            field_definition.get_data_type_ref().clone(),
            ContainerType::None,
        );
        let symbolic_struct_definition = SymbolicStructDefinition::new_anonymous(vec![scalar_field_definition]);

        Some(VirtualSnapshotQuery::Address {
            query_id: Self::query_id(project_symbol_locator, field_definition),
            address: project_symbol_locator.get_focus_address(),
            module_name: project_symbol_locator.get_focus_module_name().to_string(),
            symbolic_struct_definition,
        })
    }

    pub fn query_id(
        project_symbol_locator: &ProjectSymbolLocator,
        field_definition: &SymbolicFieldDefinition,
    ) -> String {
        format!(
            "scalar:{}:{}:{}",
            project_symbol_locator.to_locator_key(),
            field_definition.get_field_name(),
            field_definition.get_data_type_ref()
        )
    }

    pub fn deduplicate_queries_by_id(virtual_snapshot_queries: Vec<VirtualSnapshotQuery>) -> Vec<VirtualSnapshotQuery> {
        let mut seen_query_ids = HashSet::new();
        let mut deduplicated_virtual_snapshot_queries = Vec::new();

        for virtual_snapshot_query in virtual_snapshot_queries {
            if seen_query_ids.insert(virtual_snapshot_query.get_query_id().to_string()) {
                deduplicated_virtual_snapshot_queries.push(virtual_snapshot_query);
            }
        }

        deduplicated_virtual_snapshot_queries
    }

    pub fn read_integer_value(data_value: &DataValue) -> Option<i128> {
        let value_bytes = data_value.get_value_bytes();

        match data_value.get_data_type_id() {
            "u8" => value_bytes.first().map(|value_byte| i128::from(*value_byte)),
            "i8" => value_bytes
                .first()
                .map(|value_byte| i128::from(i8::from_ne_bytes([*value_byte]))),
            "u16" => Some(i128::from(u16::from_le_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "u16be" => Some(i128::from(u16::from_be_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "i16" => Some(i128::from(i16::from_le_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "i16be" => Some(i128::from(i16::from_be_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "u32" => Some(i128::from(u32::from_le_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "u32be" => Some(i128::from(u32::from_be_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "i32" => Some(i128::from(i32::from_le_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "i32be" => Some(i128::from(i32::from_be_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "u64" => Some(i128::from(u64::from_le_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "u64be" => Some(i128::from(u64::from_be_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "i64" => Some(i128::from(i64::from_le_bytes(Self::read_scalar_bytes(value_bytes)?))),
            "i64be" => Some(i128::from(i64::from_be_bytes(Self::read_scalar_bytes(value_bytes)?))),
            _ => None,
        }
    }

    fn data_type_is_supported(data_type_id: &str) -> bool {
        matches!(
            data_type_id,
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "u16be" | "u32be" | "u64be" | "i16be" | "i32be" | "i64be"
        )
    }

    fn read_scalar_bytes<const BYTE_COUNT: usize>(value_bytes: &[u8]) -> Option<[u8; BYTE_COUNT]> {
        value_bytes.get(0..BYTE_COUNT)?.try_into().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTreeScalarValue;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{container_type::ContainerType, data_value::DataValue},
        projects::project_symbol_locator::ProjectSymbolLocator,
        structs::symbolic_field_definition::SymbolicFieldDefinition,
    };
    use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;

    #[test]
    fn build_query_reads_scalar_at_resolved_locator() {
        let field_definition = SymbolicFieldDefinition::new_named(String::from("e_lfanew"), DataTypeRef::new("u32"), ContainerType::None);
        let project_symbol_locator = ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x3C);
        let scalar_query = SymbolTreeScalarValue::build_query(&project_symbol_locator, &field_definition, 4).expect("Expected scalar query.");

        let (query_id, address, module_name, symbolic_struct_definition) = match scalar_query {
            VirtualSnapshotQuery::Address {
                query_id,
                address,
                module_name,
                symbolic_struct_definition,
            } => Some((query_id, address, module_name, symbolic_struct_definition)),
            VirtualSnapshotQuery::Pointer { .. } => None,
        }
        .expect("Expected address scalar query.");

        assert_eq!(query_id, "scalar:module:game.exe:3C:e_lfanew:u32");
        assert_eq!(address, 0x3C);
        assert_eq!(module_name, "game.exe");
        assert_eq!(symbolic_struct_definition.get_fields().len(), 1);
        assert_eq!(symbolic_struct_definition.get_fields()[0].get_field_name(), "e_lfanew");
        assert_eq!(symbolic_struct_definition.get_fields()[0].get_data_type_ref(), &DataTypeRef::new("u32"));
    }

    #[test]
    fn read_integer_value_supports_endian_and_signed_integer_values() {
        let little_endian_u32 = DataValue::new(DataTypeRef::new("u32"), 0x1234_5678_u32.to_le_bytes().to_vec());
        let big_endian_i16 = DataValue::new(DataTypeRef::new("i16be"), (-2_i16).to_be_bytes().to_vec());

        assert_eq!(SymbolTreeScalarValue::read_integer_value(&little_endian_u32), Some(0x1234_5678));
        assert_eq!(SymbolTreeScalarValue::read_integer_value(&big_endian_i16), Some(-2));
    }
}
