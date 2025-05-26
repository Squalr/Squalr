use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::display_value_type::DisplayValueType;

pub struct ConversionsFromHexadecimal {}

impl ConversionsFromHexadecimal {
    pub fn convert_to_display_value(
        data_value: &str,
        to_display_value_type: DisplayValueType,
    ) -> Result<String, ConversionError> {
        match to_display_value_type {
            DisplayValueType::Binary(prepend_prefix) => Self::convert_to_binary(data_value, prepend_prefix),
            DisplayValueType::Decimal => Self::convert_to_decimal(data_value),
            DisplayValueType::Address(prepend_prefix) => Self::convert_to_address(data_value, prepend_prefix),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a hexadecimal string to a binary string.
    pub fn convert_to_binary(
        data_value: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 16, 2, prepend_prefix)
    }

    /// Converts a hexadecimal string to a decimal string.
    pub fn convert_to_decimal(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 16, 10, false)
    }

    /// Converts a hexadecimal string to an address string.
    pub fn convert_to_address(
        data_value: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(data_value, 16, prepend_prefix)
    }
}
