use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use num_traits::{Bounded, ToPrimitive};

pub struct ConversionsFromBinary {}

impl ConversionsFromBinary {
    pub fn convert_to_format(
        binary_string_to_convert: &str,
        to_anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<String, ConversionError> {
        match to_anonymous_value_string_format {
            AnonymousValueStringFormat::Decimal => Self::convert_to_decimal(binary_string_to_convert),
            AnonymousValueStringFormat::Hexadecimal => Self::convert_to_hexadecimal(binary_string_to_convert),
            AnonymousValueStringFormat::Address => Self::convert_to_address(binary_string_to_convert),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a binary string to a decimal string.
    pub fn convert_to_decimal(binary_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(binary_string_to_convert, 2, 10)
    }

    /// Converts a binary string to a hexadecimal string.
    pub fn convert_to_hexadecimal(binary_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(binary_string_to_convert, 2, 16)
    }

    /// Converts a binary string to a padded 16-character hexadecimal string (u64).
    pub fn convert_to_address(binary_string_to_convert: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(binary_string_to_convert, 2)
    }

    /// Converts a binary string to its big-endian byte representation (minimal, trimmed leading zeros).
    pub fn binary_to_bytes(binary_string_to_convert: &str) -> Result<Vec<u8>, ConversionError> {
        BaseSystemConversions::convert_to_bytes(binary_string_to_convert, 2)
    }

    /// Converts a binary string to its byte representation in the specified endianness, padded to the size of the given primitive type.
    pub fn binary_to_primitive_aligned_bytes<T: Copy + Bounded + ToPrimitive>(
        binary_string_to_convert: &str,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, ConversionError> {
        BaseSystemConversions::convert_to_primitive_aligned_bytes::<T>(binary_string_to_convert, 2, is_big_endian)
    }
}
