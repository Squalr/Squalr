use crate::conversions::conversion_error::ConversionError;
use crate::conversions::conversions_from_binary::ConversionsFromBinary;
use crate::conversions::conversions_from_hexadecimal::ConversionsFromHexadecimal;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;

pub struct PrimitiveDataTypeString {}

impl PrimitiveDataTypeString {
    pub fn get_supported_anonymous_value_string_formats() -> Vec<AnonymousValueStringFormat> {
        vec![AnonymousValueStringFormat::String]
    }

    pub fn deanonymize_string<F>(
        anonymous_value_string: &AnonymousValueString,
        decode_string_func: F,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        F: Fn(&str) -> Vec<u8>,
    {
        let bytes = match anonymous_value_string.get_anonymous_value_string_format() {
            // For binary strings, we directly map the binary to bytes.
            AnonymousValueStringFormat::Binary => ConversionsFromBinary::binary_to_bytes(&anonymous_value_string.get_anonymous_value_string())
                .map_err(|error: ConversionError| DataTypeError::ParseError(error.to_string()))?,
            // For hex strings, we directly map the hex to bytes.
            AnonymousValueStringFormat::Hexadecimal => ConversionsFromHexadecimal::hex_to_bytes(&anonymous_value_string.get_anonymous_value_string())
                .map_err(|error: ConversionError| DataTypeError::ParseError(error.to_string()))?,
            // For normal strings, we decode into the appropriate provided encoding.
            AnonymousValueStringFormat::String => decode_string_func(anonymous_value_string.get_anonymous_value_string()),
            _ => return Err(DataTypeError::ParseError("Unsupported data value format".to_string())),
        };

        Ok(bytes)
    }
}
