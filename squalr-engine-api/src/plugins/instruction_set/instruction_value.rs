use crate::{
    plugins::instruction_set::InstructionSet,
    structures::{
        data_types::{data_type_error::DataTypeError, data_type_ref::DataTypeRef},
        data_values::{
            anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
            data_value::DataValue,
        },
    },
};

pub fn deanonymize_instruction_value(
    data_type_id: &str,
    instruction_set: &dyn InstructionSet,
    anonymous_value_string: &AnonymousValueString,
) -> Result<DataValue, DataTypeError> {
    let value_bytes = match anonymous_value_string.get_anonymous_value_string_format() {
        AnonymousValueStringFormat::String => instruction_set
            .assemble(anonymous_value_string.get_anonymous_value_string())
            .map_err(DataTypeError::ParseError)?,
        AnonymousValueStringFormat::Hexadecimal => parse_hex_byte_string(anonymous_value_string.get_anonymous_value_string())?,
        unsupported_format => {
            return Err(DataTypeError::ParseError(format!(
                "Unsupported {} format '{}' for ISA data type '{}'.",
                instruction_set.get_display_name(),
                unsupported_format,
                data_type_id
            )));
        }
    };

    Ok(DataValue::new(DataTypeRef::new(data_type_id), value_bytes))
}

pub fn anonymize_instruction_bytes(
    instruction_set: &dyn InstructionSet,
    data_type_id: &str,
    value_bytes: &[u8],
    anonymous_value_string_format: AnonymousValueStringFormat,
) -> Result<AnonymousValueString, DataTypeError> {
    let anonymous_value_string = match anonymous_value_string_format {
        AnonymousValueStringFormat::String => instruction_set
            .disassemble(value_bytes)
            .map_err(DataTypeError::ParseError)?,
        AnonymousValueStringFormat::Hexadecimal => format_hex_byte_string(value_bytes),
        unsupported_format => {
            return Err(DataTypeError::ParseError(format!(
                "Unsupported {} format '{}' for ISA data type '{}'.",
                instruction_set.get_display_name(),
                unsupported_format,
                data_type_id
            )));
        }
    };

    Ok(AnonymousValueString::new(
        anonymous_value_string,
        anonymous_value_string_format,
        ContainerType::None,
    ))
}

fn parse_hex_byte_string(hex_bytes: &str) -> Result<Vec<u8>, DataTypeError> {
    let normalized_hex_bytes = hex_bytes.trim();

    if normalized_hex_bytes.is_empty() {
        return Ok(Vec::new());
    }

    let explicit_tokens: Vec<&str> = normalized_hex_bytes
        .split(|character: char| character.is_whitespace() || character == ',' || character == '-')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect();

    if explicit_tokens.len() > 1 {
        return explicit_tokens.into_iter().map(parse_hex_byte_token).collect();
    }

    let collapsed_hex_bytes = normalized_hex_bytes.replace("0x", "").replace("0X", "");

    if collapsed_hex_bytes.len() % 2 != 0 {
        return Err(DataTypeError::ParseError(format!(
            "Hex byte string '{}' must contain an even number of hexadecimal digits.",
            hex_bytes
        )));
    }

    collapsed_hex_bytes
        .as_bytes()
        .chunks_exact(2)
        .map(|hex_pair| {
            let token =
                std::str::from_utf8(hex_pair).map_err(|error| DataTypeError::ParseError(format!("Invalid UTF-8 while parsing hex bytes: {}.", error)))?;

            parse_hex_byte_token(token)
        })
        .collect()
}

fn parse_hex_byte_token(token: &str) -> Result<u8, DataTypeError> {
    let normalized_token = token.trim().trim_start_matches("0x").trim_start_matches("0X");

    if normalized_token.len() != 2 {
        return Err(DataTypeError::ParseError(format!(
            "Hex byte token '{}' must contain exactly two hexadecimal digits.",
            token
        )));
    }

    u8::from_str_radix(normalized_token, 16).map_err(|error| DataTypeError::ParseError(format!("Invalid hex byte token '{}': {}.", token, error)))
}

fn format_hex_byte_string(value_bytes: &[u8]) -> String {
    value_bytes
        .iter()
        .map(|value_byte| format!("{:02X}", value_byte))
        .collect::<Vec<_>>()
        .join(" ")
}
