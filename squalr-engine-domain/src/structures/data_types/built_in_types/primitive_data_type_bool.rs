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
        _bool_data_type_size_bytes: u64,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        Vec<u8>: From<<T as num_traits::ToBytes>::Bytes>,
    {
        let original_string = anonymous_value_string.get_anonymous_value_string();
        let normalized = original_string.trim().to_ascii_lowercase();
        let format = anonymous_value_string.get_anonymous_value_string_format();

        let parse_error = || DataTypeError::ParseError(format!("Invalid boolean string '{}' for format {:?}", original_string, format));

        let parse_decimal_like_boolean = |decimal_like_string: &str| -> Result<bool, DataTypeError> {
            if decimal_like_string.is_empty()
                || !decimal_like_string
                    .chars()
                    .all(|character| character.is_ascii_digit())
            {
                return Err(parse_error());
            }

            Ok(decimal_like_string.chars().any(|character| character != '0'))
        };

        let parse_binary_like_boolean = |binary_like_string: &str| -> Result<bool, DataTypeError> {
            if binary_like_string.is_empty()
                || !binary_like_string
                    .chars()
                    .all(|character| character == '0' || character == '1')
            {
                return Err(parse_error());
            }

            Ok(binary_like_string.chars().any(|character| character == '1'))
        };

        let parse_hex_like_boolean = |hex_like_string: &str| -> Result<bool, DataTypeError> {
            if hex_like_string.is_empty()
                || !hex_like_string
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                return Err(parse_error());
            }

            Ok(hex_like_string.chars().any(|character| character != '0'))
        };

        let boolean = match format {
            AnonymousValueStringFormat::Bool | AnonymousValueStringFormat::String => match normalized.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => parse_decimal_like_boolean(normalized.as_str()),
            },
            AnonymousValueStringFormat::Binary => {
                let binary_string = normalized.strip_prefix("0b").unwrap_or(normalized.as_str());
                parse_binary_like_boolean(binary_string)
            }
            AnonymousValueStringFormat::Decimal => parse_decimal_like_boolean(normalized.as_str()),
            AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => {
                let hexadecimal_string = normalized.strip_prefix("0x").unwrap_or(normalized.as_str());
                parse_hex_like_boolean(hexadecimal_string)
            }
            _ => Err(parse_error()),
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
