use crate::structures::{data_types::data_type_error::DataTypeError, memory::endian::Endian};

const MAX_INTEGER_BYTE_COUNT: usize = 16;

pub struct ScalarIntegerValue;

impl ScalarIntegerValue {
    pub fn read_unsigned(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<i128, DataTypeError> {
        let unsigned_value = Self::read_unsigned_u128(value_bytes, endian)?;

        i128::try_from(unsigned_value).map_err(|_| DataTypeError::ParseError(String::from("Unsigned scalar integer value does not fit in i128.")))
    }

    pub fn read_signed(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<i128, DataTypeError> {
        let unsigned_value = Self::read_unsigned_u128(value_bytes, endian)?;
        let bit_count = value_bytes.len().saturating_mul(8);

        if bit_count == MAX_INTEGER_BYTE_COUNT * 8 {
            return Ok(i128::from_ne_bytes(unsigned_value.to_ne_bytes()));
        }

        let sign_bit = 1_u128 << (bit_count - 1);
        let sign_extended_value = if unsigned_value & sign_bit == 0 {
            unsigned_value
        } else {
            unsigned_value | (!0_u128 << bit_count)
        };

        Ok(i128::from_ne_bytes(sign_extended_value.to_ne_bytes()))
    }

    fn read_unsigned_u128(
        value_bytes: &[u8],
        endian: Endian,
    ) -> Result<u128, DataTypeError> {
        Self::validate_byte_count(value_bytes)?;

        let mut padded_value_bytes = [0_u8; MAX_INTEGER_BYTE_COUNT];

        match endian {
            Endian::Little => padded_value_bytes[..value_bytes.len()].copy_from_slice(value_bytes),
            Endian::Big => {
                let start_byte_index = MAX_INTEGER_BYTE_COUNT - value_bytes.len();
                padded_value_bytes[start_byte_index..].copy_from_slice(value_bytes);
            }
        }

        Ok(match endian {
            Endian::Little => u128::from_le_bytes(padded_value_bytes),
            Endian::Big => u128::from_be_bytes(padded_value_bytes),
        })
    }

    fn validate_byte_count(value_bytes: &[u8]) -> Result<(), DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        if value_bytes.len() > MAX_INTEGER_BYTE_COUNT {
            return Err(DataTypeError::InvalidByteCount {
                expected: MAX_INTEGER_BYTE_COUNT as u64,
                actual: value_bytes.len() as u64,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ScalarIntegerValue;
    use crate::structures::memory::endian::Endian;

    #[test]
    fn reads_unsigned_little_endian_non_power_of_two_byte_width() {
        let value = ScalarIntegerValue::read_unsigned(&[0x56, 0x34, 0x12], Endian::Little).expect("Expected scalar integer value to decode.");

        assert_eq!(value, 0x12_3456);
    }

    #[test]
    fn reads_signed_big_endian_non_power_of_two_byte_width() {
        let value = ScalarIntegerValue::read_signed(&[0xFF, 0x80, 0x00], Endian::Big).expect("Expected scalar integer value to decode.");

        assert_eq!(value, -32768);
    }
}
