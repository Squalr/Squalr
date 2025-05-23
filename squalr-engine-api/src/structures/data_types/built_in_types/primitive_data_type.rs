use crate::structures::data_types::built_in_types::primitive_display_type::PrimitiveDisplayType;
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
            AnonymousValueContainer::StringValue(value_string, _) => {
                let normalized = value_string.trim().to_ascii_lowercase();
                let boolean = match normalized.as_str() {
                    "true" | "1" => true,
                    "false" | "0" => false,
                    _ => return Err(DataTypeError::ParseError(format!("Invalid boolean string '{}'", value_string))),
                };

                let primitive: T = if boolean { T::from(1) } else { T::from(0) };
                let bytes = if is_big_endian { primitive.to_be_bytes() } else { primitive.to_le_bytes() };

                Ok(DataValue::new(data_type_ref, bytes.into()))
            }

            AnonymousValueContainer::ByteArray(value_bytes) => {
                let expected_size = std::mem::size_of::<T>();
                if value_bytes.len() != expected_size {
                    return Err(DataTypeError::InvalidByteCount {
                        expected: expected_size,
                        actual: value_bytes.len(),
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
            AnonymousValueContainer::StringValue(value_string, is_value_hex) => {
                if *is_value_hex {
                    match Conversions::hex_to_primitive_bytes::<T>(&value_string, is_big_endian) {
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
                    }
                } else {
                    match value_string.parse::<T>() {
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
                    }
                }
            }
            AnonymousValueContainer::ByteArray(value_bytes) => Ok(DataValue::new(data_type_ref, value_bytes.clone())),
        }
    }

    pub fn create_display_value<T, F>(
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
        convert_bytes_unchecked: F,
    ) -> Result<String, DataTypeError>
    where
        F: Fn() -> T,
        T: ToString,
    {
        let expected = std::mem::size_of::<T>();
        let actual = value_bytes.len();

        match data_type_meta_data {
            DataTypeMetaData::Primitive(primitive_display_type) => {
                if actual == expected {
                    let value = convert_bytes_unchecked();
                    let value_string = value.to_string();
                    match primitive_display_type {
                        PrimitiveDisplayType::AsHex => Conversions::dec_to_hex(&value_string, false)
                            .map_err(|err| DataTypeError::ParseError(format!("Error converting primitive to hex: {}", err))),
                        PrimitiveDisplayType::AsAddress => Conversions::dec_to_address(&value_string, false)
                            .map_err(|err| DataTypeError::ParseError(format!("Error converting primitive to hex: {}", err))),
                        PrimitiveDisplayType::Normal => Ok(value_string),
                    }
                } else {
                    Err(DataTypeError::InvalidByteCount { expected, actual })
                }
            }
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    pub fn get_meta_data_for_anonymous_value(anonymous_value: &AnonymousValue) -> DataTypeMetaData {
        match anonymous_value.get_value() {
            AnonymousValueContainer::StringValue(_, is_hex) => {
                if *is_hex {
                    DataTypeMetaData::Primitive(PrimitiveDisplayType::AsHex)
                } else {
                    DataTypeMetaData::Primitive(PrimitiveDisplayType::Normal)
                }
            }
            AnonymousValueContainer::ByteArray(_) => DataTypeMetaData::Primitive(PrimitiveDisplayType::Normal),
        }
    }
}
