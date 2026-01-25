use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::data_value_interpretation_format::DataValueInterpretationFormat;

pub struct ConversionsFromHexadecimal {}

impl ConversionsFromHexadecimal {
    pub fn convert_to_data_value_interpreter(
        data_value: &str,
        to_data_value_interpretation_format: DataValueInterpretationFormat,
    ) -> Result<String, ConversionError> {
        match to_data_value_interpretation_format {
            DataValueInterpretationFormat::Binary => Self::convert_to_binary(data_value),
            DataValueInterpretationFormat::Decimal => Self::convert_to_decimal(data_value),
            DataValueInterpretationFormat::Address => Self::convert_to_address(data_value),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a hexadecimal string to a binary string.
    pub fn convert_to_binary(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 16, 2)
    }

    /// Converts a hexadecimal string to a decimal string.
    pub fn convert_to_decimal(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 16, 10)
    }

    /// Converts a hexadecimal string to an address string.
    pub fn convert_to_address(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_address(data_value, 16)
    }
}
