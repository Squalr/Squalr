use crate::conversions::base_system_conversions::BaseSystemConversions;
use crate::conversions::conversion_error::ConversionError;
use crate::structures::data_values::data_value_interpretation_format::DataValueInterpretationFormat;

pub struct ConversionsFromDecimal {}

impl ConversionsFromDecimal {
    pub fn convert_to_data_value_interpreter(
        data_value: &str,
        to_data_value_interpretation_format: DataValueInterpretationFormat,
    ) -> Result<String, ConversionError> {
        match to_data_value_interpretation_format {
            DataValueInterpretationFormat::Binary => Self::convert_to_binary(&data_value),
            DataValueInterpretationFormat::Hexadecimal => Self::convert_to_hexadecimal(&data_value),
            DataValueInterpretationFormat::Address => Self::convert_to_address(&data_value),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a decimal string to a binary string.
    pub fn convert_to_binary(data_value: &str) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 10, 2)
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
