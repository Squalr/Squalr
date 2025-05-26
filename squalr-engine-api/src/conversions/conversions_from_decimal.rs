use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::display_value_type::DisplayValueType;

pub struct ConversionsFromDecimal {}

impl ConversionsFromDecimal {
    pub fn convert_to_display_value(
        data_value: &str,
        to_display_value_type: DisplayValueType,
    ) -> Result<String, ConversionError> {
        match to_display_value_type {
            DisplayValueType::Binary(display_container) => Self::convert_to_binary(&data_value),
            DisplayValueType::Hexadecimal(display_container) => Self::convert_to_hexadecimal(&data_value),
            DisplayValueType::Address(display_container) => Self::convert_to_address(&data_value),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a decimal string to a binary string.
    pub fn convert_to_binary(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 16, 2)
    }

    /// Converts a decimal string to a hexadecimal string.
    pub fn convert_to_hexadecimal(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 10, 16)
    }

    /// Converts a decimal string to an 8 or 16-character hexadecimal string (u64).
    pub fn convert_to_address(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(data_value, 10)
    }
}
