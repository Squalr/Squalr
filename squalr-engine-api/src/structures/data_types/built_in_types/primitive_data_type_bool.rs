use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::container_type::ContainerType;

pub struct PrimitiveDataTypeBool {}

impl PrimitiveDataTypeBool {
    pub fn get_supported_anonymous_value_string_formats_bool() -> Vec<AnonymousValueStringFormat> {
        vec![
            AnonymousValueStringFormat::Bool,
            AnonymousValueStringFormat::Binary,
            AnonymousValueStringFormat::Decimal,
            AnonymousValueStringFormat::Hexadecimal,
        ]
    }

    pub fn deanonymize<T: Copy + num_traits::ToBytes + From<u8>>(
        anonymous_value_string: &AnonymousValueString,
        is_big_endian: bool,
        bool_data_type_size_bytes: u64,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        Vec<u8>: From<<T as num_traits::ToBytes>::Bytes>,
    {
        let original_string = anonymous_value_string.get_anonymous_value_string();
        let normalized = original_string.trim().to_ascii_lowercase();
        let max_leading_zeros = (bool_data_type_size_bytes * 8) - 1;
        let is_valid = |string: &str| -> bool {
            if string == "0" || string == "1" {
                return true;
            }
            if string.starts_with('0') && string.len() - 1 <= max_leading_zeros as usize {
                return string[1..].chars().all(|char| char == '0' || char == '1');
            }
            false
        };
        let boolean = match anonymous_value_string.get_anonymous_value_string_format() {
            AnonymousValueStringFormat::Bool | AnonymousValueStringFormat::String => {
                if is_valid(&normalized) {
                    Ok(normalized
                        .trim_start_matches('0')
                        .parse::<bool>()
                        .unwrap_or(false))
                } else {
                    Err(DataTypeError::ParseError(format!(
                        "Invalid boolean string '{}' for format {:?}",
                        original_string,
                        anonymous_value_string.get_anonymous_value_string_format()
                    )))
                }
            }
            AnonymousValueStringFormat::Binary
            | AnonymousValueStringFormat::Decimal
            | AnonymousValueStringFormat::Hexadecimal
            | AnonymousValueStringFormat::Address => {
                if is_valid(&normalized) {
                    Ok(normalized
                        .trim_start_matches('0')
                        .parse::<bool>()
                        .unwrap_or(false))
                } else {
                    Err(DataTypeError::ParseError(format!(
                        "Invalid boolean string '{}' for format {:?}",
                        original_string,
                        anonymous_value_string.get_anonymous_value_string_format()
                    )))
                }
            }
            _ => Err(DataTypeError::ParseError(format!(
                "Invalid boolean string '{}' for format {:?}",
                original_string,
                anonymous_value_string.get_anonymous_value_string_format()
            ))),
        }?;

        let primitive: T = if boolean { T::from(1) } else { T::from(0) };
        let bytes = if is_big_endian { primitive.to_be_bytes() } else { primitive.to_le_bytes() };

        Ok(bytes.into())
    }

    pub fn anonymize<T: Copy + num_traits::ToBytes + From<u8>>(
        value_bytes: &[u8],
        _is_big_endian: bool,
        anonymous_value_string_format: AnonymousValueStringFormat,
        bool_primitive_size: u64,
    ) -> Result<AnonymousValueString, DataTypeError> {
        let element_size = bool_primitive_size as usize;
        let array_size = value_bytes.len() % element_size;
        let container_type = if array_size > 1 {
            ContainerType::ArrayFixed(array_size as u64)
        } else {
            ContainerType::None
        };
        let mut bool_strings = vec![];

        for chunk in value_bytes.chunks_exact(element_size) {
            // We can actually ignore is_big_endian because we really only care if any value is non-zero, which is invariant across endianness.
            let is_true = chunk.iter().any(|&byte| byte != 0);

            bool_strings.push(if is_true { "true" } else { "false" });
        }

        let anonymous_value_string = bool_strings.join(", ");

        Ok(AnonymousValueString::new(anonymous_value_string, anonymous_value_string_format, container_type))
    }
}
