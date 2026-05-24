use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use num_traits::{Bounded, ToPrimitive};

pub struct ConversionsFromDecimal {}

impl ConversionsFromDecimal {
    pub fn convert_to_format(
        decimal_string_to_convert: &str,
        to_anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<String, ConversionError> {
        match to_anonymous_value_string_format {
            AnonymousValueStringFormat::Binary => Self::convert_to_binary(decimal_string_to_convert),
            AnonymousValueStringFormat::Hexadecimal => Self::convert_to_hexadecimal(decimal_string_to_convert),
            AnonymousValueStringFormat::Address => Self::convert_to_address(decimal_string_to_convert),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a decimal string to a binary string.
    pub fn convert_to_binary(decimal_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(decimal_string_to_convert, 10, 2)
    }

    /// Converts a decimal string to a hexadecimal string.
    pub fn convert_to_hexadecimal(decimal_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(decimal_string_to_convert, 10, 16)
    }

    /// Converts a decimal string to an 8 or 16-character hexadecimal string (u64).
    pub fn convert_to_address(decimal_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(decimal_string_to_convert, 10)
    }

    /// Converts a decimal string to its big-endian byte representation (minimal, trimmed leading zeros).
    pub fn decimal_to_bytes(decimal_string_to_convert: &str) -> Result<Vec<u8>, ConversionError> {
        BaseSystemConversions::convert_to_bytes(decimal_string_to_convert, 10)
    }

    /// Converts a decimal string to its byte representation in the specified endianness, padded to the size of the given primitive type.
    pub fn decimal_to_primitive_aligned_bytes<T: Copy + Bounded + ToPrimitive>(
        decimal_string_to_convert: &str,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, ConversionError> {
        BaseSystemConversions::convert_to_primitive_aligned_bytes::<T>(decimal_string_to_convert, 10, is_big_endian)
    }
}
