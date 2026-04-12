use crate::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use std::str::FromStr;

pub fn normalize_instruction_data_type_id(symbolic_field_definition_text: &str) -> Option<String> {
    let symbolic_field_definition = SymbolicFieldDefinition::from_str(symbolic_field_definition_text).ok()?;
    let data_type_id = symbolic_field_definition
        .get_data_type_ref()
        .get_data_type_id()
        .trim();

    if data_type_id.starts_with("i_") {
        Some(data_type_id.to_string())
    } else {
        None
    }
}

pub fn is_instruction_data_type_id(symbolic_field_definition_text: &str) -> bool {
    normalize_instruction_data_type_id(symbolic_field_definition_text).is_some()
}

#[cfg(test)]
mod tests {
    use super::{is_instruction_data_type_id, normalize_instruction_data_type_id};

    #[test]
    fn normalize_instruction_data_type_id_extracts_base_data_type_id() {
        assert_eq!(normalize_instruction_data_type_id("i_x86[7]").as_deref(), Some("i_x86"));
    }

    #[test]
    fn normalize_instruction_data_type_id_rejects_non_instruction_types() {
        assert_eq!(normalize_instruction_data_type_id("u32"), None);
        assert!(!is_instruction_data_type_id("u32[4]"));
    }
}
