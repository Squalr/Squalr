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
            DisplayValueType::Decimal(container_type) => Self::convert_to_decimal(&data_value),
            DisplayValueType::Hexadecimal(container_type) => Self::convert_to_hexadecimal(&data_value),
            DisplayValueType::Address(container_type) => Self::convert_to_address(&data_value),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a binary string to a decimal string.
    pub fn convert_to_decimal(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 2, 10)
    }

    /// Converts a binary string to a hexadecimal string.
    pub fn convert_to_hexadecimal(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 2, 16)
    }

    /// Converts a binary string to a padded 16-character hexadecimal string (u64).
    pub fn convert_to_address(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(data_value, 2)
    }
}
