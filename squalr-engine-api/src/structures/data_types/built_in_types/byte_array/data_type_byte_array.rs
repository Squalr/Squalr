use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::{AnonymousValue, AnonymousValueContainer};
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};
use squalr_engine_common::conversions::Conversions;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeByteArray {}

impl DataTypeByteArray {
    pub const DATA_TYPE_ID: &str = "byte_array";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_value_from_primitive(value_bytes: &[u8]) -> DataValue {
        DataValue::new(
            DataTypeRef::new(Self::get_data_type_id(), DataTypeMetaData::SizedContainer(value_bytes.len() as u64)),
            value_bytes.to_vec(),
        )
    }
}

impl DataType for DataTypeByteArray {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        1
    }

    fn validate_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> bool {
        let data_type_ref = DataTypeRef::new_from_anonymous_value(self.get_data_type_id(), anonymous_value);

        match self.deanonymize_value(anonymous_value, data_type_ref) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
        data_type_ref: DataTypeRef,
    ) -> Result<DataValue, DataTypeError> {
        if data_type_ref.get_data_type_id() != Self::get_data_type_id() {
            return Err(DataTypeError::InvalidDataType);
        }

        match anonymous_value.get_value() {
            AnonymousValueContainer::StringValue(value_string, is_value_hex) => {
                if *is_value_hex {
                    let value_bytes = Conversions::hex_to_bytes(value_string).map_err(|err: &str| DataTypeError::ParseError(err.to_string()))?;

                    // Group into bytes (2 hex digits each).
                    Ok(DataValue::new(data_type_ref, value_bytes))
                } else {
                    // For decimal, allow space or comma separation.
                    let value_bytes = value_string
                        .split(|next_char: char| next_char.is_whitespace() || next_char == ',')
                        .filter(|next_value| !next_value.is_empty())
                        .map(|next_value| {
                            u8::from_str_radix(next_value, 10).map_err(|err| DataTypeError::ValueParseError {
                                value: next_value.to_string(),
                                is_value_hex: false,
                                source: err,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(DataValue::new(data_type_ref, value_bytes))
                }
            }
            AnonymousValueContainer::ByteArray(value_bytes) => Ok(DataValue::new(data_type_ref, value_bytes.clone())),
        }
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
        _data_type_meta_data: &DataTypeMetaData,
    ) -> Result<String, DataTypeError> {
        if !value_bytes.is_empty() {
            Ok(value_bytes
                .iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<String>>()
                .join(" "))
        } else {
            Err(DataTypeError::NoBytes)
        }
    }

    fn is_discrete(&self) -> bool {
        true
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        let array_size = match data_type_ref.get_meta_data() {
            DataTypeMetaData::SizedContainer(size) => *size as usize,
            _ => {
                log::error!("Invalid metadata provided to byte array data type.");
                0usize
            }
        };

        DataValue::new(data_type_ref, vec![0u8; array_size])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::SizedContainer(1)
    }

    fn get_meta_data_for_anonymous_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> DataTypeMetaData {
        let byte_count = match anonymous_value.get_value() {
            AnonymousValueContainer::StringValue(value_string, is_value_hex) => {
                if *is_value_hex {
                    Conversions::hex_to_bytes(value_string)
                        .unwrap_or_default()
                        .len()
                } else {
                    // For decimal, allow space or comma separation.
                    value_string
                        .split(|next_char: char| next_char.is_whitespace() || next_char == ',')
                        .filter(|next_value| !next_value.is_empty())
                        .map(|next_value| u8::from_str_radix(next_value, 10))
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap_or_default()
                        .len()
                }
            }
            AnonymousValueContainer::ByteArray(value_bytes) => value_bytes.len(),
        } as u64;
        DataTypeMetaData::SizedContainer(byte_count)
    }

    fn get_meta_data_from_string(
        &self,
        string: &str,
    ) -> Result<DataTypeMetaData, String> {
        let container_size = match string.parse::<u64>() {
            Ok(container_size) => container_size,
            Err(err) => {
                return Err(format!("Failed to parse container size: {}", err));
            }
        };

        Ok(DataTypeMetaData::SizedContainer(container_size))
    }
}
