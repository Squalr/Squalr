use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use num_traits::{Bounded, ToPrimitive};

pub struct ConversionsFromHexadecimal {}

impl ConversionsFromHexadecimal {
    pub fn convert_to_format(
        hex_string_to_convert: &str,
        to_anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<String, ConversionError> {
        match to_anonymous_value_string_format {
            AnonymousValueStringFormat::Binary => Self::convert_to_binary(hex_string_to_convert),
            AnonymousValueStringFormat::Decimal => Self::convert_to_decimal(hex_string_to_convert),
            AnonymousValueStringFormat::Address => Self::convert_to_address(hex_string_to_convert),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a hexadecimal string to a binary string.
    pub fn convert_to_binary(hex_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(hex_string_to_convert, 16, 2)
    }

    /// Converts a hexadecimal string to a decimal string.
    pub fn convert_to_decimal(hex_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(hex_string_to_convert, 16, 10)
    }

    /// Converts a hexadecimal string to an address string.
    pub fn convert_to_address(hex_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(hex_string_to_convert, 16)
    }

    /// Converts a decimal string to its big-endian byte representation (minimal, trimmed leading zeros).
    pub fn hex_to_bytes(hex_string_to_convert: &str) -> Result<Vec<u8>, ConversionError> {
        BaseSystemConversions::convert_to_bytes(hex_string_to_convert, 16)
    }

    /// Converts a hexadecimal string to its byte representation in the specified endianness, padded to the size of the given primitive type.
    pub fn hex_to_primitive_aligned_bytes<T: Copy + Bounded + ToPrimitive>(
        hex_string_to_convert: &str,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, ConversionError> {
        BaseSystemConversions::convert_to_primitive_aligned_bytes::<T>(hex_string_to_convert, 16, is_big_endian)
    }
}
