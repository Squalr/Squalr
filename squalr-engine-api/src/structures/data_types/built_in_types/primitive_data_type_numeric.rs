use crate::conversions::conversions_from_binary::ConversionsFromBinary;
use crate::conversions::conversions_from_hexadecimal::ConversionsFromHexadecimal;
use crate::conversions::conversions_from_primitives::Conversions;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::container_type::ContainerType;
use num_traits::{Bounded, ToBytes, ToPrimitive};
use std::fmt;
use std::{any::type_name, mem::size_of, str::FromStr};

pub trait AsBits {
    /// An integer type that already implements all the formatting traits we need.
    type Bits: fmt::Binary + fmt::UpperHex + fmt::Display + ToString;

    /// Return the raw bit pattern of `self`.
    fn as_bits(&self) -> Self::Bits;
}

macro_rules! impl_as_bits_int {
    ($($t:ty)*) => {$(
        impl AsBits for $t {
            type Bits = $t;

            fn as_bits(&self) -> Self::Bits { *self }
        }
    )*};
}

impl_as_bits_int!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

impl AsBits for f32 {
    type Bits = u32;

    fn as_bits(&self) -> Self::Bits {
        self.to_bits()
    }
}

impl AsBits for f64 {
    type Bits = u64;

    fn as_bits(&self) -> Self::Bits {
        self.to_bits()
    }
}

pub struct PrimitiveDataTypeNumeric {}

impl PrimitiveDataTypeNumeric {
    pub fn get_supported_anonymous_value_string_formats() -> Vec<AnonymousValueStringFormat> {
        vec![
            AnonymousValueStringFormat::Binary,
            AnonymousValueStringFormat::Decimal,
            AnonymousValueStringFormat::Hexadecimal,
        ]
    }

    pub fn deanonymize<T: Copy + FromStr + ToBytes + Bounded + ToPrimitive>(
        anonymous_value_string: &AnonymousValueString,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        T::Bytes: Into<Vec<u8>>,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let primitive_size = size_of::<T>();
        let value_string = anonymous_value_string.get_anonymous_value_string();
        let has_multiple_elements = matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_));

        if !has_multiple_elements {
            return Self::deanonymize_single::<T>(value_string, anonymous_value_string.get_anonymous_value_string_format(), is_big_endian);
        }

        let value_parts = value_string
            .split(|character: char| character == ',' || character.is_whitespace())
            .map(|value_part| value_part.trim())
            .filter(|value_part| !value_part.is_empty())
            .collect::<Vec<_>>();

        if value_parts.is_empty() {
            return Err(DataTypeError::ParseError("Numeric array scans require at least one value.".to_string()));
        }

        if let ContainerType::ArrayFixed(expected_value_count) = anonymous_value_string.get_container_type() {
            if value_parts.len() as u64 != expected_value_count {
                return Err(DataTypeError::ParseError(format!(
                    "Expected {} values for array input, but found {}.",
                    expected_value_count,
                    value_parts.len()
                )));
            }
        }

        let mut combined_value_bytes = Vec::with_capacity(value_parts.len() * primitive_size);

        for value_part in value_parts {
            let value_bytes = Self::deanonymize_single::<T>(value_part, anonymous_value_string.get_anonymous_value_string_format(), is_big_endian)?;

            combined_value_bytes.extend(value_bytes);
        }

        Ok(combined_value_bytes)
    }

    pub fn anonymize<T: Copy + num_traits::ToBytes + From<u8>, F>(
        value_bytes: &[u8],
        convert_bytes_unchecked: F,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError>
    where
        F: Fn(&[u8]) -> T,
        T: AsBits + ToString + fmt::Display,
    {
        let element_size = std::mem::size_of::<T>();
        let mut result_strings = vec![];

        for chunk in value_bytes.chunks_exact(element_size) {
            let value = convert_bytes_unchecked(chunk);
            let bits = value.as_bits();

            match anonymous_value_string_format {
                AnonymousValueStringFormat::Binary => {
                    result_strings.push(Conversions::primitive_to_binary(&bits));
                }
                AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => {
                    result_strings.push(Conversions::primitive_to_hexadecimal(&bits));
                }
                _ => {
                    result_strings.push(value.to_string());
                }
            }
        }

        let value_string = result_strings.join(", ");
        let container_type = if result_strings.len() > 1 {
            ContainerType::ArrayFixed(result_strings.len() as u64)
        } else {
            ContainerType::None
        };

        Ok(AnonymousValueString::new(value_string, anonymous_value_string_format, container_type))
    }

    fn deanonymize_single<T: Copy + FromStr + ToBytes + Bounded + ToPrimitive>(
        value_string: &str,
        anonymous_value_string_format: AnonymousValueStringFormat,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        T::Bytes: Into<Vec<u8>>,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let primitive_size = size_of::<T>();

        match anonymous_value_string_format {
            AnonymousValueStringFormat::Binary => match ConversionsFromBinary::binary_to_primitive_aligned_bytes::<T>(&value_string, is_big_endian) {
                Ok(value_bytes) => {
                    if value_bytes.len() < primitive_size {
                        return Err(DataTypeError::ParseError(format!(
                            "Failed to decode binary bytes '{}'. Length is {} bytes, but expected at least {} bytes for {}.",
                            value_string,
                            value_bytes.len(),
                            primitive_size,
                            type_name::<T>()
                        )));
                    }

                    Ok(value_bytes)
                }
                Err(error) => Err(DataTypeError::ParseError(format!("Failed to parse binary value '{}': {}", value_string, error))),
            },
            AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => {
                match ConversionsFromHexadecimal::hex_to_primitive_aligned_bytes::<T>(&value_string, is_big_endian) {
                    Ok(value_bytes) => {
                        if value_bytes.len() < primitive_size {
                            return Err(DataTypeError::ParseError(format!(
                                "Failed to decode hex bytes '{}'. Length is {} bytes, but expected at least {} bytes for {}.",
                                value_string,
                                value_bytes.len(),
                                primitive_size,
                                type_name::<T>()
                            )));
                        }

                        Ok(value_bytes)
                    }
                    Err(error) => Err(DataTypeError::ParseError(format!("Failed to parse hex value '{}': {}", value_string, error))),
                }
            }
            _ => match value_string.parse::<T>() {
                Ok(value) => {
                    if is_big_endian {
                        Ok(value.to_be_bytes().into())
                    } else {
                        Ok(value.to_le_bytes().into())
                    }
                }
                Err(error) => Err(DataTypeError::ParseError(format!(
                    "Failed to parse {} value '{}': {}",
                    type_name::<T>(),
                    value_string,
                    error
                ))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveDataTypeNumeric;
    use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
    use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use crate::structures::data_values::container_type::ContainerType;

    #[test]
    fn deanonymize_supports_address_format_using_little_endian_bytes() {
        let anonymous_value_string = AnonymousValueString::new(String::from("0x3010"), AnonymousValueStringFormat::Address, ContainerType::None);

        let value_bytes =
            PrimitiveDataTypeNumeric::deanonymize::<u64>(&anonymous_value_string, false).expect("Expected the address-formatted numeric value to parse.");

        assert_eq!(value_bytes, 0x3010_u64.to_le_bytes());
    }

    #[test]
    fn deanonymize_supports_decimal_array_values_when_container_type_is_array() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2, 3"), AnonymousValueStringFormat::Decimal, ContainerType::Array);

        let value_bytes = PrimitiveDataTypeNumeric::deanonymize::<u32>(&anonymous_value_string, false).expect("Expected decimal array input to parse.");

        assert_eq!(
            value_bytes,
            [
                1_u32.to_le_bytes().as_slice(),
                2_u32.to_le_bytes().as_slice(),
                3_u32.to_le_bytes().as_slice(),
            ]
            .concat()
        );
    }

    #[test]
    fn deanonymize_rejects_decimal_array_values_when_container_type_is_element() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2, 3"), AnonymousValueStringFormat::Decimal, ContainerType::None);

        let result = PrimitiveDataTypeNumeric::deanonymize::<u32>(&anonymous_value_string, false);

        assert!(result.is_err());
    }

    #[test]
    fn deanonymize_rejects_mismatched_fixed_array_length() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2"), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(3));

        let result = PrimitiveDataTypeNumeric::deanonymize::<u16>(&anonymous_value_string, false);

        assert!(result.is_err());
    }
}
