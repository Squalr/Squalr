use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeByteArray {}

impl DataTypeByteArray {
    pub fn get_data_type_id() -> &'static str {
        &"byte_array"
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

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Result<Vec<u8>, DataTypeError> {
        let value_string = anonymous_value.to_string();
        let base = if anonymous_value.is_value_hex { 16 } else { 10 };

        if anonymous_value.is_value_hex {
            // Clean input: remove whitespace, commas, and any 0x prefixes.
            let cleaned = value_string
                .replace(|next_char: char| next_char.is_whitespace() || next_char == ',', "")
                .replace("0x", "")
                .replace("0X", "");

            if cleaned.len() % 2 != 0 {
                return Err(DataTypeError::ParseError(format!("Hex string has an odd number of digits: '{}'", cleaned)));
            }

            // Group into bytes (2 hex digits each).
            cleaned
                .as_bytes()
                .chunks(2)
                .map(|chunk| {
                    let hex_str = std::str::from_utf8(chunk).unwrap_or_default();
                    u8::from_str_radix(hex_str, 16).map_err(|err| DataTypeError::ValueParseError {
                        value: hex_str.to_string(),
                        is_value_hex: true,
                        source: err,
                    })
                })
                .collect()
        } else {
            // For decimal, allow space or comma separation.
            value_string
                .split(|next_char: char| next_char.is_whitespace() || next_char == ',')
                .filter(|next_value| !next_value.is_empty())
                .map(|next_value| {
                    u8::from_str_radix(next_value, base).map_err(|err| DataTypeError::ValueParseError {
                        value: next_value.to_string(),
                        is_value_hex: false,
                        source: err,
                    })
                })
                .collect()
        }
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
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

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn get_default_value(
        &self,
        data_type_meta_data: &DataTypeMetaData,
    ) -> DataValue {
        let array_size = match data_type_meta_data {
            DataTypeMetaData::SizedContainer(size) => *size as usize,
            _ => {
                log::error!("Invalid metadata provided to byte array data type.");
                0usize
            }
        };

        DataValue::new(self.get_ref(), vec![0u8; array_size])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::SizedContainer(1)
    }
}
