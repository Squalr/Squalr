use crate::conversions::conversion_error::ConversionError;
use crate::conversions::conversions_from_binary::ConversionsFromBinary;
use crate::conversions::conversions_from_hexadecimal::ConversionsFromHexadecimal;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_values::anonymous_value_bytes::AnonymousValueBytes;
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

pub struct PrimitiveDataType {}

impl PrimitiveDataType {
    pub fn deanonymize_bool<T: Copy + num_traits::ToBytes + From<u8>>(
        anonymous_value_string: &AnonymousValueString,
        is_big_endian: bool,
        bool_data_type_size_bytes: u64,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        Vec<u8>: From<<T as num_traits::ToBytes>::Bytes>,
    {
        let original_string = anonymous_value_string.get_anonymous_value_string();
        let normalized = original_string.trim().to_ascii_lowercase();
        let max_leading_zeros = (bool_data_type_size_bytes * 8) - 1;
        let is_valid = |string: &str| -> bool {
            if string == "0" || string == "1" {
                return true;
            }
            if string.starts_with('0') && string.len() - 1 <= max_leading_zeros as usize {
                return string[1..].chars().all(|char| char == '0' || char == '1');
            }
            false
        };
        let boolean = match anonymous_value_string.get_anonymous_value_string_format() {
            AnonymousValueStringFormat::Bool | AnonymousValueStringFormat::String => {
                if is_valid(&normalized) {
                    Ok(normalized
                        .trim_start_matches('0')
                        .parse::<bool>()
                        .unwrap_or(false))
                } else {
                    Err(DataTypeError::ParseError(format!(
                        "Invalid boolean string '{}' for format {:?}",
                        original_string,
                        anonymous_value_string.get_anonymous_value_string_format()
                    )))
                }
            }
            AnonymousValueStringFormat::Binary
            | AnonymousValueStringFormat::Decimal
            | AnonymousValueStringFormat::Hexadecimal
            | AnonymousValueStringFormat::Address => {
                if is_valid(&normalized) {
                    Ok(normalized
                        .trim_start_matches('0')
                        .parse::<bool>()
                        .unwrap_or(false))
                } else {
                    Err(DataTypeError::ParseError(format!(
                        "Invalid boolean string '{}' for format {:?}",
                        original_string,
                        anonymous_value_string.get_anonymous_value_string_format()
                    )))
                }
            }
            _ => Err(DataTypeError::ParseError(format!(
                "Invalid boolean string '{}' for format {:?}",
                original_string,
                anonymous_value_string.get_anonymous_value_string_format()
            ))),
        }?;

        let primitive: T = if boolean { T::from(1) } else { T::from(0) };
        let bytes = if is_big_endian { primitive.to_be_bytes() } else { primitive.to_le_bytes() };

        Ok(bytes.into())
    }

    pub fn deanonymize_primitive_value_string<T: Copy + FromStr + ToBytes + Bounded + ToPrimitive>(
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

    pub fn deanonymize_primitive_value_bytes<T: std::str::FromStr + Copy + num_traits::ToBytes>(
        anonymous_value_bytes: &AnonymousValueBytes,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        T::Bytes: Into<Vec<u8>>,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let primitive_size = size_of::<T>();

        match anonymous_value_bytes.get_container_type() {
            ContainerType::None => {
                let mut result_bytes = anonymous_value_bytes.get_value().to_vec();

                if is_big_endian {
                    result_bytes.reverse();
                }

                Ok(result_bytes)
            }
            ContainerType::Array => {
                let bytes = anonymous_value_bytes.get_value();
                let mut result_bytes = bytes.to_vec();

                if is_big_endian {
                    for chunk in result_bytes.chunks_exact_mut(primitive_size) {
                        chunk.reverse();
                    }
                }

                Ok(result_bytes)
            }
            ContainerType::ArrayFixed(array_length) => {
                let value_bytes = anonymous_value_bytes.get_value();

                if value_bytes.len() != array_length as usize * primitive_size {
                    return Err(DataTypeError::ParseError(format!(
                        "Fixed array length mismatch: expected {} bytes, got {} bytes",
                        array_length as usize * primitive_size,
                        value_bytes.len()
                    )));
                }

                let mut result_bytes = value_bytes.to_vec();

                if is_big_endian {
                    for chunk in result_bytes.chunks_exact_mut(primitive_size) {
                        chunk.reverse();
                    }
                }

                Ok(result_bytes)
            }
            ContainerType::Pointer32 | ContainerType::Pointer64 => Err(DataTypeError::UnsupportedContainerType {
                container_type: anonymous_value_bytes.get_container_type(),
            }),
        }
    }

    pub fn decode_string<F>(
        anonymous_value_string: &AnonymousValueString,
        decode_string_func: F,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        F: Fn(&str) -> Vec<u8>,
    {
        let bytes = match anonymous_value_string.get_anonymous_value_string_format() {
            // For binary strings, we directly map the binary to bytes.
            AnonymousValueStringFormat::Binary => ConversionsFromBinary::binary_to_bytes(&anonymous_value_string.get_anonymous_value_string())
                .map_err(|error: ConversionError| DataTypeError::ParseError(error.to_string()))?,
            // For hex strings, we directly map the hex to bytes.
            AnonymousValueStringFormat::Hexadecimal => ConversionsFromHexadecimal::hex_to_bytes(&anonymous_value_string.get_anonymous_value_string())
                .map_err(|error: ConversionError| DataTypeError::ParseError(error.to_string()))?,
            // For normal strings, we decode into the appropriate provided encoding.
            AnonymousValueStringFormat::String => decode_string_func(anonymous_value_string.get_anonymous_value_string()),
            _ => return Err(DataTypeError::ParseError("Unsupported data value format".to_string())),
        };

        Ok(bytes)
    }

    /*
    pub fn create_data_value_interpreters<T, F>(
        value_bytes: &[u8],
        convert_bytes_unchecked: F,
    ) -> Result<DataValueInterpreters, DataTypeError>
    where
        F: Fn(&[u8]) -> T,
        T: AsBits + ToString + fmt::Display,
    {
        let element_size = std::mem::size_of::<T>();
        let mut binary_strings = vec![];
        let mut decimal_strings = vec![];
        let mut hexadecimal_strings = vec![];

        for chunk in value_bytes.chunks_exact(element_size) {
            let value = convert_bytes_unchecked(chunk);
            let bits = value.as_bits();

            binary_strings.push(Conversions::primitive_to_binary(&bits));
            decimal_strings.push(value.to_string());
            hexadecimal_strings.push(Conversions::primitive_to_hexadecimal(&bits));
        }

        let value_string_binary = binary_strings.join(", ");
        let value_string_decimal = decimal_strings.join(", ");
        let value_string_hexadecimal = hexadecimal_strings.join(", ");
        let mut results = vec![];

        for supported_display_type in Self::get_supported_anonymous_value_string_formats() {
            match supported_display_type {
                AnonymousValueStringFormat::Binary => results.push(DataValueInterpreter::new(
                    value_string_binary.clone(),
                    supported_display_type,
                    ContainerType::None,
                )),
                AnonymousValueStringFormat::Decimal => results.push(DataValueInterpreter::new(
                    value_string_decimal.clone(),
                    supported_display_type,
                    ContainerType::None,
                )),
                AnonymousValueStringFormat::Hexadecimal => results.push(DataValueInterpreter::new(
                    value_string_hexadecimal.clone(),
                    supported_display_type,
                    ContainerType::None,
                )),
                _ => {
                    log::error!("Unhandled supported primitive display type!")
                }
            };
        }

        Ok(DataValueInterpreters::new(results, AnonymousValueStringFormat::Decimal))
    }

    pub fn create_data_value_interpreters_bool(
        value_bytes: &[u8],
        bool_primitive_size: u64,
    ) -> Result<DataValueInterpreters, DataTypeError> {
        let element_size = bool_primitive_size as usize;
        let mut bool_strings = vec![];

        for chunk in value_bytes.chunks_exact(element_size) {
            let is_true = chunk.iter().any(|&byte| byte != 0);

            bool_strings.push(if is_true { "true" } else { "false" });
        }

        let value_string_bool = bool_strings.join(", ");

        Ok(DataValueInterpreters::new(
            vec![DataValueInterpreter::new(
                value_string_bool,
                AnonymousValueStringFormat::Bool,
                ContainerType::None,
            )],
            AnonymousValueStringFormat::Bool,
        ))
    } */

    pub fn get_supported_anonymous_value_string_formats_bool() -> Vec<AnonymousValueStringFormat> {
        vec![
            AnonymousValueStringFormat::Bool,
            AnonymousValueStringFormat::Binary,
            AnonymousValueStringFormat::Decimal,
            AnonymousValueStringFormat::Hexadecimal,
        ]
    }

    pub fn get_supported_anonymous_value_string_formats() -> Vec<AnonymousValueStringFormat> {
        vec![
            AnonymousValueStringFormat::Binary,
            AnonymousValueStringFormat::Decimal,
            AnonymousValueStringFormat::Hexadecimal,
        ]
    }
}
