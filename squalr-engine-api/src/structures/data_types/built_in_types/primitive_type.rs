use crate::structures::{
    data_types::{data_type_error::DataTypeError, data_type_ref::DataTypeRef},
    data_values::{
        anonymous_value::{AnonymousValue, AnonymousValueContainer},
        data_value::DataValue,
    },
};
use squalr_engine_common::conversions::Conversions;
use std::{any::type_name, mem::size_of, str::FromStr};

pub struct PrimitiveDataType {}

impl PrimitiveDataType {
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
}
