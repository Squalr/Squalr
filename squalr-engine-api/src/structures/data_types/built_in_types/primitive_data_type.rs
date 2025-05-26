use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::{
    data_types::{data_type_error::DataTypeError, data_type_meta_data::DataTypeMetaData, data_type_ref::DataTypeRef},
    data_values::{
        anonymous_value::{AnonymousValue, AnonymousValueContainer},
        data_value::DataValue,
    },
};
use squalr_engine_common::conversions::Conversions;
use std::{any::type_name, mem::size_of, str::FromStr};

pub struct PrimitiveDataType {}

impl PrimitiveDataType {
    pub fn deanonymize_bool<T: Copy + num_traits::ToBytes + From<u8>>(
        anonymous_value: &AnonymousValue,
        data_type_ref: DataTypeRef,
        is_big_endian: bool,
    ) -> Result<DataValue, DataTypeError>
    where
        Vec<u8>: From<<T as num_traits::ToBytes>::Bytes>,
    {
        match anonymous_value.get_value() {
            AnonymousValueContainer::BinaryValue(value_string)
            | AnonymousValueContainer::HexValue(value_string)
            | AnonymousValueContainer::StringValue(value_string) => {
                let normalized = value_string.trim().to_ascii_lowercase();
                let boolean = match anonymous_value.get_value() {
                    AnonymousValueContainer::StringValue(_) => match normalized.to_lowercase().as_str() {
                        "true" | "1" => true,
                        "false" | "0" => false,
                        _ => return Err(DataTypeError::ParseError(format!("Invalid boolean string '{}'", value_string))),
                    },
                    // For binary and hex, we only support '0'/'1' as the proper encoding for a bool.
                    _ => match normalized.as_str() {
                        "1" => true,
                        "0" => false,
                        _ => return Err(DataTypeError::ParseError(format!("Invalid boolean string '{}'", value_string))),
                    },
                };

                let primitive: T = if boolean { T::from(1) } else { T::from(0) };
                let bytes = if is_big_endian { primitive.to_be_bytes() } else { primitive.to_le_bytes() };

                Ok(DataValue::new(data_type_ref, bytes.into()))
            }
            AnonymousValueContainer::ByteArray(value_bytes) => {
                let expected_size = std::mem::size_of::<T>() as u64;
                let actual_size = value_bytes.len() as u64;

                if actual_size != expected_size {
                    return Err(DataTypeError::InvalidByteCount {
                        expected: expected_size,
                        actual: actual_size,
                    });
                }

                // Any non-zero byte value is interpreted as true.
                let is_true = value_bytes.iter().any(|&b| b != 0);
                let primitive: T = if is_true { T::from(1) } else { T::from(0) };
                let bytes = if is_big_endian { primitive.to_be_bytes() } else { primitive.to_le_bytes() };

                Ok(DataValue::new(data_type_ref, bytes.into()))
            }
        }
    }

    pub fn deanonymize_primitive<T: std::str::FromStr + Copy + num_traits::ToBytes>(
        anonymous_value: &AnonymousValue,
        data_type_ref: DataTypeRef,
        is_big_endian: bool,
    ) -> Result<DataValue, DataTypeError>
    where
        T::Bytes: Into<Vec<u8>>,
        <T as FromStr>::Err: std::fmt::Display,
    {
        match anonymous_value.get_value() {
            AnonymousValueContainer::BinaryValue(value_string) => match Conversions::binary_to_primitive_bytes::<T>(&value_string, is_big_endian) {
                Ok(bytes) => {
                    if bytes.len() < size_of::<T>() {
                        return Err(DataTypeError::ParseError(format!(
                            "Failed to decode hex bytes '{}'. Length is {} bytes, but expected at least {} bytes for {}.",
                            value_string,
                            bytes.len(),
                            size_of::<T>(),
                            type_name::<T>()
                        )));
                    }
                    Ok(DataValue::new(data_type_ref, bytes))
                }
                Err(err) => Err(DataTypeError::ParseError(format!("Failed to parse hex value '{}': {}", value_string, err))),
            },
            AnonymousValueContainer::HexValue(value_string) => match Conversions::hex_to_primitive_bytes::<T>(&value_string, is_big_endian) {
                Ok(bytes) => {
                    if bytes.len() < size_of::<T>() {
                        return Err(DataTypeError::ParseError(format!(
                            "Failed to decode hex bytes '{}'. Length is {} bytes, but expected at least {} bytes for {}.",
                            value_string,
                            bytes.len(),
                            size_of::<T>(),
                            type_name::<T>()
                        )));
                    }
                    Ok(DataValue::new(data_type_ref, bytes))
                }
                Err(err) => Err(DataTypeError::ParseError(format!("Failed to parse hex value '{}': {}", value_string, err))),
            },
            AnonymousValueContainer::StringValue(value_string) => match value_string.parse::<T>() {
                Ok(value) => {
                    let bytes = if is_big_endian { value.to_be_bytes() } else { value.to_le_bytes() };
                    Ok(DataValue::new(data_type_ref, bytes.into()))
                }
                Err(err) => Err(DataTypeError::ParseError(format!(
                    "Failed to parse {} value '{}': {}",
                    type_name::<T>(),
                    value_string,
                    err
                ))),
            },
            AnonymousValueContainer::ByteArray(value_bytes) => Ok(DataValue::new(data_type_ref, value_bytes.clone())),
        }
    }

    pub fn create_display_values<T, F>(
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
        convert_bytes_unchecked: F,
    ) -> Result<DisplayValues, DataTypeError>
    where
        F: Fn() -> T,
        T: ToString,
    {
        match data_type_meta_data {
            DataTypeMetaData::Primitive() => {
                let expected = std::mem::size_of::<T>() as u64;
                let actual = value_bytes.len() as u64;

                if actual == expected {
                    let mut results = vec![];
                    let value = convert_bytes_unchecked();
                    let value_string = value.to_string();

                    match Conversions::dec_to_hex(&value_string, false) {
                        Ok(display_value) => results.push(DisplayValue::new(DisplayValueType::Hex, display_value)),
                        Err(err) => log::error!("Error converting primitive to hex display value: {}", err),
                    }

                    match Conversions::dec_to_address(&value_string, false) {
                        Ok(display_value) => results.push(DisplayValue::new(DisplayValueType::Address, display_value)),
                        Err(err) => log::error!("Error converting primitive to address display value: {}", err),
                    }

                    results.push(DisplayValue::new(DisplayValueType::Dec, value_string));

                    Ok(DisplayValues::new(results, DisplayValueType::Dec))
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
                            vec![DisplayValue::new(DisplayValueType::Bool, "false".into())],
                            DisplayValueType::Bool,
                        ))
                    } else {
                        // For our impl we consider non-zero to be true.
                        Ok(DisplayValues::new(
                            vec![DisplayValue::new(DisplayValueType::Bool, "true".into())],
                            DisplayValueType::Bool,
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
            DisplayValueType::Bin,
            DisplayValueType::Dec,
            DisplayValueType::Hex,
        ]
    }
}
