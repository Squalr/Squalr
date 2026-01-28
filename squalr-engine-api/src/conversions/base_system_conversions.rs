use crate::conversions::conversion_error::ConversionError;
use num_traits::{Bounded, ToPrimitive};

pub struct BaseSystemConversions {}

impl BaseSystemConversions {
    /// Converts a string from one base system to another. Supports up to a 128-bit integer.
    pub fn convert_to_base(
        value_string_to_convert: &str,
        from_base: u32,
        to_base: u32,
    ) -> Result<String, ConversionError> {
        let value_128 = u128::from_str_radix(value_string_to_convert, from_base)?;

        let result = match to_base {
            2 => {
                format!("{:b}", value_128)
            }
            10 => format!("{}", value_128),
            16 => {
                format!("{:x}", value_128)
            }
            _ => return Err(ConversionError::UnsupportedConversion),
        };

        Ok(result)
    }

    /// Converts a value to a padded hexadecimal address. Supports up to a 128-bit integer.
    pub fn convert_to_address(
        value_string_to_convert: &str,
        from_base: u32,
    ) -> Result<String, ConversionError> {
        u128::from_str_radix(value_string_to_convert, from_base)
            .map(|parsed_value| {
                if parsed_value <= u32::MAX as u128 {
                    format!("{:08x}", parsed_value)
                } else if parsed_value <= u64::MAX as u128 {
                    format!("{:016x}", parsed_value)
                } else {
                    format!("{:032x}", parsed_value)
                }
            })
            .map_err(ConversionError::from)
    }

    /// Converts a base-N string to its big-endian byte representation (minimal, trimmed leading zeros).
    pub fn convert_to_bytes(
        value_string: &str,
        from_base: u32,
    ) -> Result<Vec<u8>, ConversionError> {
        let cleaned = Self::clean_string(value_string, from_base);

        if cleaned.is_empty() {
            return Ok(Vec::new());
        }

        let value = u128::from_str_radix(&cleaned, from_base).map_err(ConversionError::from)?;
        let mut bytes = value.to_be_bytes().to_vec();
        let first_non_zero = bytes
            .iter()
            .position(|&byte| byte != 0)
            .unwrap_or(bytes.len() - 1);

        bytes.drain(0..first_non_zero);

        if bytes.is_empty() {
            bytes.push(0);
        }

        Ok(bytes)
    }

    /// Converts a base-N string to its byte representation in the specified endianness, padded to the size of the given primitive type.
    pub fn convert_to_primitive_aligned_bytes<T: Bounded + ToPrimitive>(
        value_string: &str,
        from_base: u32,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, ConversionError> {
        let cleaned = Self::clean_string(value_string, from_base);

        let value = if cleaned.is_empty() {
            0u128
        } else {
            u128::from_str_radix(&cleaned, from_base).map_err(ConversionError::from)?
        };

        let max_value = T::max_value()
            .to_u128()
            .ok_or(ConversionError::UnsupportedConversion)?;

        if value > max_value {
            return Err(ConversionError::UnsupportedConversion);
        }

        let size = std::mem::size_of::<T>();

        // Convert to minimal big-endian byte representation.
        let bytes = if value == 0 {
            vec![0]
        } else {
            let mut bytes = value.to_be_bytes().to_vec();

            while bytes.first() == Some(&0) {
                bytes.remove(0);
            }

            bytes
        };

        // Value must fit into T.
        if bytes.len() > size {
            return Err(ConversionError::UnsupportedConversion);
        }

        // Pad with zeros.
        let mut padded = vec![0u8; size - bytes.len()];

        padded.extend(bytes);

        // Swap endian if requested.
        if is_big_endian {
            padded.reverse();
        }

        Ok(padded)
    }

    fn clean_string(
        value_string: &str,
        from_base: u32,
    ) -> String {
        let mut value_string = value_string.to_string();
        let value_string_lower = value_string.to_lowercase();

        if (from_base == 16 && value_string_lower.starts_with("0x")) || (from_base == 2 && value_string_lower.starts_with("0b")) {
            value_string = value_string[2..].to_string();
        }

        value_string
            .chars()
            .filter(|char| !char.is_whitespace() && *char != ',')
            .collect()
    }
}
