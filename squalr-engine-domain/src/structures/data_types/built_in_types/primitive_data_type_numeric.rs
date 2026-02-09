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
        let value_bytes = match anonymous_value_string.get_anonymous_value_string_format() {
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
                    value_bytes
                }
                Err(error) => {
                    return Err(DataTypeError::ParseError(format!("Failed to parse binary value '{}': {}", value_string, error)));
                }
            },
            AnonymousValueStringFormat::Hexadecimal => match ConversionsFromHexadecimal::hex_to_primitive_aligned_bytes::<T>(&value_string, is_big_endian) {
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
                    value_bytes
                }
                Err(error) => {
                    return Err(DataTypeError::ParseError(format!("Failed to parse hex value '{}': {}", value_string, error)));
                }
            },
            _ => match value_string.parse::<T>() {
                Ok(value) => {
                    if is_big_endian {
                        value.to_be_bytes().into()
                    } else {
                        value.to_le_bytes().into()
                    }
                }
                Err(error) => {
                    return Err(DataTypeError::ParseError(format!(
                        "Failed to parse {} value '{}': {}",
                        type_name::<T>(),
                        value_string,
                        error
                    )));
                }
            },
        };

        Ok(value_bytes)
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
}
