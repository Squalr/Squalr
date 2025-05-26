use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::display_value_type::DisplayValueType;

pub struct ConversionsFromBinary {}

impl ConversionsFromBinary {
    pub fn convert_to_display_value(
        data_value: &str,
        to_display_value_type: DisplayValueType,
    ) -> Result<String, ConversionError> {
        match to_display_value_type {
            DisplayValueType::Decimal => Self::convert_to_decimal(&data_value),
            DisplayValueType::Hexadecimal(prepend_prefix) => Self::convert_to_hexadecimal(&data_value, prepend_prefix),
            DisplayValueType::Address(prepend_prefix) => Self::convert_to_address(&data_value, prepend_prefix),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a binary string to a decimal string.
    pub fn convert_to_decimal(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 2, 10, false)
    }

    /// Converts a binary string to a hexadecimal string.
    pub fn convert_to_hexadecimal(
        data_value: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 2, 16, prepend_prefix)
    }

    /// Converts a binary string to a padded 16-character hexadecimal string (u64).
    pub fn convert_to_address(
        data_value: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(data_value, 2, prepend_prefix)
    }
}
