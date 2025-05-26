use crate::conversions::conversion_error::ConversionError;

pub struct BaseSystemConversions {}

impl BaseSystemConversions {
    /// Converts a string from one base system to another.
    pub fn convert_to_base(
        value: &str,
        from_base: u32,
        to_base: u32,
    ) -> Result<String, ConversionError> {
        let val = u64::from_str_radix(value, from_base)?;

        let result = match to_base {
            2 => {
                format!("{:b}", val)
            }
            10 => format!("{}", val),
            16 => {
                format!("{:x}", val)
            }
            _ => return Err(ConversionError::UnsupportedConversion),
        };

        Ok(result)
    }

    /// Converts a value to a padded hexadecimal address.
    pub fn convert_to_address(
        value: &str,
        from_base: u32,
    ) -> Result<String, ConversionError> {
        u64::from_str_radix(value, from_base)
            .map(|val| {
                if val <= u32::MAX as u64 {
                    format!("{:08x}", val)
                } else {
                    format!("{:016x}", val)
                }
            })
            .map_err(ConversionError::from)
    }
}
