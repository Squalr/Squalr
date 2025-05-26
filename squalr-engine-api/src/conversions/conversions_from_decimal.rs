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
            DisplayValueType::Binary(prepend_prefix) => Self::convert_to_binary(&data_value, prepend_prefix),
            DisplayValueType::Hexadecimal(prepend_prefix) => Self::convert_to_hexadecimal(&data_value, prepend_prefix),
            DisplayValueType::Address(prepend_prefix) => Self::convert_to_address(&data_value, prepend_prefix),
            _ => Err(ConversionError::UnsupportedConversion),
        }
    }

    /// Converts a decimal string to a binary string.
    pub fn convert_to_binary(
        data_value: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        BaseSystemConversions::convert_to_base(data_value, 16, 2, prepend_prefix)
    }

    /// Converts a decimal string to a hexadecimal string.
    pub fn convert_to_hexadecimal(
        data_value: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        // Parse the decimal string to a u64 string.
        u64::from_str_radix(data_value, 2)
            .map(|val| if prepend_prefix { format!("0x{:x}", val) } else { format!("{:x}", val) })
            .map_err(ConversionError::from)
    }

    /// Converts a decimal string to an 8 or 16-character hexadecimal string (u64).
    pub fn convert_to_address(
        dec: &str,
        prepend_prefix: bool,
    ) -> Result<String, ConversionError> {
        dec.parse::<u64>()
            .map(|val| {
                let hex = if val <= u32::MAX as u64 {
                    format!("{:08x}", val)
                } else {
                    format!("{:016x}", val)
                };
                if prepend_prefix { format!("0x{}", hex) } else { hex }
            })
            .map_err(ConversionError::from)
    }
}
