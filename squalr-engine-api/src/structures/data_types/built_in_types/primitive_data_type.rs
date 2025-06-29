use crate::conversions::conversions::Conversions;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::{
    data_types::{data_type_error::DataTypeError, data_type_meta_data::DataTypeMetaData},
    data_values::anonymous_value::AnonymousValueContainer,
};
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
        anonymous_value_container: &AnonymousValueContainer,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        Vec<u8>: From<<T as num_traits::ToBytes>::Bytes>,
    {
        // Generally this is one iteration, but in the case where doing an array scan, we concat all the values together.
        let boolean = match anonymous_value_container {
            AnonymousValueContainer::BinaryValue(value_string) | AnonymousValueContainer::HexadecimalValue(value_string) => {
                let normalized = value_string.trim().to_ascii_lowercase();
                // For binary and hex, we only support '0'/'1' as the proper encoding for a bool.
                match normalized.as_str() {
                    "1" => true,
                    "0" => false,
                    _ => return Err(DataTypeError::ParseError(format!("Invalid boolean string '{}'", value_string))),
                }
            }
            AnonymousValueContainer::String(value_string) => {
                let normalized = value_string.trim().to_ascii_lowercase();
                match normalized.to_lowercase().as_str() {
                    "true" | "1" => true,
                    "false" | "0" => false,
                    _ => return Err(DataTypeError::ParseError(format!("Invalid boolean string '{}'", value_string))),
                }
            }
        };

        let primitive: T = if boolean { T::from(1) } else { T::from(0) };
        let bytes = if is_big_endian { primitive.to_be_bytes() } else { primitive.to_le_bytes() };

        Ok(bytes.into())
    }

    pub fn deanonymize_primitive<T: std::str::FromStr + Copy + num_traits::ToBytes>(
        anonymous_value_container: &AnonymousValueContainer,
        is_big_endian: bool,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        T::Bytes: Into<Vec<u8>>,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let bytes = match anonymous_value_container {
            AnonymousValueContainer::BinaryValue(value_string) => match Conversions::binary_to_primitive_bytes::<T>(&value_string, is_big_endian) {
                Ok(val_bytes) => {
                    if val_bytes.len() < size_of::<T>() {
                        return Err(DataTypeError::ParseError(format!(
                            "Failed to decode binary bytes '{}'. Length is {} bytes, but expected at least {} bytes for {}.",
                            value_string,
                            val_bytes.len(),
                            size_of::<T>(),
                            type_name::<T>()
                        )));
                    }
                    val_bytes
                }
                Err(err) => {
                    return Err(DataTypeError::ParseError(format!("Failed to parse binary value '{}': {}", value_string, err)));
                }
            },
            AnonymousValueContainer::HexadecimalValue(value_string) => match Conversions::hex_to_primitive_bytes::<T>(&value_string, is_big_endian) {
                Ok(val_bytes) => {
                    if val_bytes.len() < size_of::<T>() {
                        return Err(DataTypeError::ParseError(format!(
                            "Failed to decode hex bytes '{}'. Length is {} bytes, but expected at least {} bytes for {}.",
                            value_string,
                            val_bytes.len(),
                            size_of::<T>(),
                            type_name::<T>()
                        )));
                    }
                    val_bytes
                }
                Err(err) => {
                    return Err(DataTypeError::ParseError(format!("Failed to parse hex value '{}': {}", value_string, err)));
                }
            },
            AnonymousValueContainer::String(value_string) => match value_string.parse::<T>() {
                Ok(value) => {
                    if is_big_endian {
                        value.to_be_bytes().into()
                    } else {
                        value.to_le_bytes().into()
                    }
                }
                Err(err) => {
                    return Err(DataTypeError::ParseError(format!(
                        "Failed to parse {} value '{}': {}",
                        type_name::<T>(),
                        value_string,
                        err
                    )));
                }
            },
        };

        Ok(bytes)
    }

    pub fn decode_string<F>(
        anonymous_value_container: &AnonymousValueContainer,
        data_type_ref: &DataTypeRef,
        decode_string_func: F,
    ) -> Result<Vec<u8>, DataTypeError>
    where
        F: Fn(&String) -> Vec<u8>,
    {
        match data_type_ref.get_meta_data() {
            DataTypeMetaData::SizedContainer(size) => {
                let mut bytes = match anonymous_value_container {
                    // For binary strings, we directly map the binary to bytes.
                    AnonymousValueContainer::BinaryValue(value_string_utf8) => {
                        Conversions::binary_to_bytes(&value_string_utf8).map_err(|err: &str| DataTypeError::ParseError(err.to_string()))?
                    }
                    // For hex strings, we directly map the hex to bytes.
                    AnonymousValueContainer::HexadecimalValue(value_string_utf8) => {
                        Conversions::hex_to_bytes(&value_string_utf8).map_err(|err: &str| DataTypeError::ParseError(err.to_string()))?
                    }
                    // For normal strings, we decode into the appropriate provided encoding.
                    AnonymousValueContainer::String(value_string_utf8) => decode_string_func(value_string_utf8),
                };

                // Truncate to container size.
                bytes.truncate(*size as usize);

                Ok(bytes)
            }
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    /// Merges an array of data values of the same data type into a singular data value.
    pub fn array_merge(data_values: Vec<DataValue>) -> Result<DataValue, DataTypeError> {
        if let Some(merged_values) = data_values.first() {
            let merged_data_type = merged_values.get_data_type();
            let mut merged_data_type_meta_data = merged_data_type.get_meta_data().clone();
            let mut merged_bytes = merged_values.get_value_bytes().clone();

            for next in data_values.iter().skip(1) {
                if merged_values.get_data_type_id() != next.get_data_type_id() {
                    return Err(DataTypeError::DataValueMergeError {
                        error: "Data type mismatch in array merge!".to_string(),
                    });
                }

                let mut next_bytes = next.get_value_bytes().clone();

                merged_bytes.append(&mut next_bytes);

                merged_data_type_meta_data = match merged_data_type_meta_data {
                    DataTypeMetaData::SizedContainer(size) => match next.get_data_type().get_meta_data() {
                        DataTypeMetaData::SizedContainer(next_size) => DataTypeMetaData::SizedContainer(size + *next_size),
                        _ => {
                            return Err(DataTypeError::DataValueMergeError {
                                error: "Mismatched data type metadata in array merge!".to_string(),
                            });
                        }
                    },
                    _ => merged_data_type_meta_data,
                };
            }

            return Ok(DataValue::new(
                DataTypeRef::new(merged_data_type.get_data_type_id(), merged_data_type_meta_data),
                merged_bytes,
            ));
        }

        Err(DataTypeError::DataValueMergeError {
            error: "No values provided to array merge!".to_string(),
        })
    }

    pub fn create_display_values<T, F>(
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
        convert_bytes_unchecked: F,
    ) -> Result<DisplayValues, DataTypeError>
    where
        F: Fn() -> T,
        T: AsBits + ToString + fmt::Display,
    {
        match data_type_meta_data {
            DataTypeMetaData::Primitive() => {
                let expected = std::mem::size_of::<T>() as u64;
                let actual = value_bytes.len() as u64;

                if actual == expected {
                    let mut results = vec![];
                    let value = convert_bytes_unchecked();
                    let bits = value.as_bits();

                    let value_string_binary = Conversions::primitive_to_binary(&bits);
                    let value_string_decimal = value.to_string();
                    let value_string_hexadecimal = Conversions::primitive_to_hexadecimal(&bits);

                    for supported_display_type in Self::get_supported_display_types() {
                        match supported_display_type {
                            DisplayValueType::Binary(_) => results.push(DisplayValue::new(supported_display_type, value_string_binary.clone())),
                            DisplayValueType::Decimal(_) => results.push(DisplayValue::new(supported_display_type, value_string_decimal.clone())),
                            DisplayValueType::Hexadecimal(_) => results.push(DisplayValue::new(supported_display_type, value_string_hexadecimal.clone())),
                            _ => {
                                log::error!("Unhandled supported primitive display type!")
                            }
                        };
                    }

                    Ok(DisplayValues::new(results, DisplayValueType::Decimal(ContainerType::None)))
                } else {
                    Err(DataTypeError::InvalidByteCount { expected, actual })
                }
            }
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    pub fn create_display_values_bool(
        value_bytes: &[u8],
        default_size_in_bytes: u64,
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<DisplayValues, DataTypeError> {
        match data_type_meta_data {
            DataTypeMetaData::Primitive() => {
                let expected = default_size_in_bytes;
                let actual = value_bytes.len() as u64;

                if actual == expected {
                    if value_bytes[0] == 0 {
                        Ok(DisplayValues::new(
                            vec![DisplayValue::new(
                                DisplayValueType::Bool(ContainerType::None),
                                "false".into(),
                            )],
                            DisplayValueType::Bool(ContainerType::None),
                        ))
                    } else {
                        // For our impl we consider non-zero to be true.
                        Ok(DisplayValues::new(
                            vec![DisplayValue::new(
                                DisplayValueType::Bool(ContainerType::None),
                                "true".into(),
                            )],
                            DisplayValueType::Bool(ContainerType::None),
                        ))
                    }
                } else {
                    Err(DataTypeError::InvalidByteCount { expected, actual })
                }
            }
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    pub fn get_supported_display_types() -> Vec<DisplayValueType> {
        vec![
            DisplayValueType::Binary(ContainerType::None),
            DisplayValueType::Binary(ContainerType::Array),
            DisplayValueType::Decimal(ContainerType::None),
            DisplayValueType::Decimal(ContainerType::Array),
            DisplayValueType::Hexadecimal(ContainerType::None),
            DisplayValueType::Hexadecimal(ContainerType::Array),
        ]
    }
}
