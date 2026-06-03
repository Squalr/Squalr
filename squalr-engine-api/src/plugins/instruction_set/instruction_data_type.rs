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

pub fn instruction_set_id_from_instruction_data_type_id(instruction_data_type_id: &str) -> Option<String> {
    instruction_data_type_id
        .trim()
        .strip_prefix("i_")
        .map(str::trim)
        .filter(|instruction_set_id| !instruction_set_id.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::{instruction_set_id_from_instruction_data_type_id, is_instruction_data_type_id, normalize_instruction_data_type_id};

    #[test]
    fn normalize_instruction_data_type_id_extracts_base_data_type_id() {
        assert_eq!(normalize_instruction_data_type_id("i_x86[7]").as_deref(), Some("i_x86"));
        assert_eq!(normalize_instruction_data_type_id("i_arm64[4]").as_deref(), Some("i_arm64"));
        assert_eq!(normalize_instruction_data_type_id("i_ppc32be").as_deref(), Some("i_ppc32be"));
    }

    #[test]
    fn normalize_instruction_data_type_id_rejects_non_instruction_types() {
        assert_eq!(normalize_instruction_data_type_id("u32"), None);
        assert!(!is_instruction_data_type_id("u32[4]"));
    }

    #[test]
    fn instruction_set_id_from_instruction_data_type_id_uses_instruction_type_convention() {
        assert_eq!(instruction_set_id_from_instruction_data_type_id("i_x64").as_deref(), Some("x64"));
        assert_eq!(instruction_set_id_from_instruction_data_type_id("i_ppc32be").as_deref(), Some("ppc32be"));
        assert_eq!(instruction_set_id_from_instruction_data_type_id("u32"), None);
    }
}
