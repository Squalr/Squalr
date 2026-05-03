use squalr_engine_api::structures::data_types::built_in_types::{
    bool8::data_type_bool8::DataTypeBool8, bool32::data_type_bool32::DataTypeBool32, f32::data_type_f32::DataTypeF32, f32be::data_type_f32be::DataTypeF32be,
    f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be, i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16,
    i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32, i32be::data_type_i32be::DataTypeI32be, i64::data_type_i64::DataTypeI64,
    i64be::data_type_i64be::DataTypeI64be, string::utf8::data_type_string_utf8::DataTypeStringUtf8, u8::data_type_u8::DataTypeU8,
    u16::data_type_u16::DataTypeU16, u16be::data_type_u16be::DataTypeU16be, u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be,
    u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
};
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use std::str::FromStr;

const DATA_TYPE_ID_U24: &str = "u24";
const DATA_TYPE_ID_U24BE: &str = "u24be";
const DATA_TYPE_ID_I24: &str = "i24";
const DATA_TYPE_ID_I24BE: &str = "i24be";
const DATA_TYPE_ID_I_X86: &str = "i_x86";
const DATA_TYPE_ID_I_X64: &str = "i_x64";

pub struct DataTypeToStringConverter {}

impl DataTypeToStringConverter {
    pub fn convert_data_type_to_string(data_type_id: &str) -> String {
        let parsed_symbolic_field_definition = SymbolicFieldDefinition::from_str(data_type_id).ok();
        let normalized_data_type_id = parsed_symbolic_field_definition
            .as_ref()
            .map(|symbolic_field_definition| {
                symbolic_field_definition
                    .get_data_type_ref()
                    .get_data_type_id()
                    .to_string()
            })
            .unwrap_or_else(|| data_type_id.to_string());

        let normalized_data_type_label = match normalized_data_type_id.as_str() {
            DataTypeBool8::DATA_TYPE_ID => String::from("bool8"),
            DataTypeBool32::DATA_TYPE_ID => String::from("bool32"),
            DataTypeU8::DATA_TYPE_ID => String::from("u8"),
            DataTypeU16::DATA_TYPE_ID => String::from("u16"),
            DataTypeU16be::DATA_TYPE_ID => String::from("u16be"),
            DATA_TYPE_ID_U24 => String::from("u24"),
            DATA_TYPE_ID_U24BE => String::from("u24be"),
            DataTypeU32::DATA_TYPE_ID => String::from("u32"),
            DataTypeU32be::DATA_TYPE_ID => String::from("u32be"),
            DataTypeU64::DATA_TYPE_ID => String::from("u64"),
            DataTypeU64be::DATA_TYPE_ID => String::from("u64be"),
            DataTypeI8::DATA_TYPE_ID => String::from("i8"),
            DataTypeI16::DATA_TYPE_ID => String::from("i16"),
            DataTypeI16be::DATA_TYPE_ID => String::from("i16be"),
            DATA_TYPE_ID_I24 => String::from("i24"),
            DATA_TYPE_ID_I24BE => String::from("i24be"),
            DataTypeI32::DATA_TYPE_ID => String::from("i32"),
            DataTypeI32be::DATA_TYPE_ID => String::from("i32be"),
            DataTypeI64::DATA_TYPE_ID => String::from("i64"),
            DataTypeI64be::DATA_TYPE_ID => String::from("i64be"),
            DataTypeF32::DATA_TYPE_ID => String::from("f32"),
            DataTypeF32be::DATA_TYPE_ID => String::from("f32be"),
            DataTypeF64::DATA_TYPE_ID => String::from("f64"),
            DataTypeF64be::DATA_TYPE_ID => String::from("f64be"),
            DataTypeStringUtf8::DATA_TYPE_ID => String::from("String (UTF-8)"),
            DATA_TYPE_ID_I_X86 => String::from("i_x86"),
            DATA_TYPE_ID_I_X64 => String::from("i_x64"),
            _ => normalized_data_type_id,
        };

        if let Some(symbolic_field_definition) = parsed_symbolic_field_definition {
            let container_type = symbolic_field_definition.get_container_type();

            if container_type != ContainerType::None {
                return format!("{}{}", normalized_data_type_label, container_type);
            }
        }

        normalized_data_type_label
    }
}

#[cfg(test)]
mod tests {
    use super::DataTypeToStringConverter;

    #[test]
    fn convert_data_type_to_string_preserves_container_suffix() {
        assert_eq!(DataTypeToStringConverter::convert_data_type_to_string("u8[4]"), "u8[4]");
        assert_eq!(
            DataTypeToStringConverter::convert_data_type_to_string("player.stats*(u64)"),
            "player.stats*(u64)"
        );
    }
}
