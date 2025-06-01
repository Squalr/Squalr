use crate::conversions::conversions::Conversions;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::{AnonymousValue, AnonymousValueContainer};
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeStringUtf8 {}

impl DataTypeStringUtf8 {
    pub const DATA_TYPE_ID: &str = "string_utf8";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_value_from_primitive(str: &str) -> DataValue {
        let value_bytes = str.as_bytes();
        DataValue::new(
            DataTypeRef::new(Self::get_data_type_id(), DataTypeMetaData::SizedContainer(value_bytes.len() as u64)),
            value_bytes.to_vec(),
        )
    }
}

impl DataType for DataTypeStringUtf8 {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        "string"
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        0
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
            return Err(DataTypeError::InvalidDataTypeRef {
                data_type_ref: data_type_ref.get_data_type_id().to_string(),
            });
        }

        match data_type_ref.get_meta_data() {
            DataTypeMetaData::SizedContainer(size) => match anonymous_value.get_value() {
                AnonymousValueContainer::BinaryValue(value_string_utf8) => {
                    let mut value_bytes = Conversions::binary_to_bytes(value_string_utf8).map_err(|err: &str| DataTypeError::ParseError(err.to_string()))?;

                    value_bytes.truncate(*size as usize);

                    return Ok(DataValue::new(data_type_ref, value_bytes));
                }
                AnonymousValueContainer::HexadecimalValue(value_string_utf8) => {
                    let mut value_bytes = Conversions::hex_to_bytes(value_string_utf8).map_err(|err: &str| DataTypeError::ParseError(err.to_string()))?;

                    value_bytes.truncate(*size as usize);

                    return Ok(DataValue::new(data_type_ref, value_bytes));
                }
                AnonymousValueContainer::String(value_string_utf8) => {
                    let mut string_bytes = value_string_utf8.as_bytes().to_vec();

                    string_bytes.truncate(*size as usize);

                    Ok(DataValue::new(data_type_ref, string_bytes))
                }
            },
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn create_display_values(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<DisplayValues, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        match data_type_meta_data {
            DataTypeMetaData::SizedContainer(size) => {
                let mut value_bytes_vec = value_bytes.to_vec();

                value_bytes_vec.truncate(*size as usize);

                let decoded_string = std::str::from_utf8(&value_bytes_vec)
                    .map_err(|_err| DataTypeError::DecodingError)?
                    .to_string();

                Ok(DisplayValues::new(
                    vec![DisplayValue::new(
                        DisplayValueType::String(ContainerType::None),
                        decoded_string,
                    )],
                    DisplayValueType::String(ContainerType::None),
                ))
            }
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn get_supported_display_types(&self) -> Vec<DisplayValueType> {
        vec![
            DisplayValueType::String(ContainerType::None),
            DisplayValueType::String(ContainerType::Array),
        ]
    }

    fn get_default_display_type(&self) -> DisplayValueType {
        DisplayValueType::String(ContainerType::None)
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
        DataValue::new(data_type_ref.clone(), vec![])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::SizedContainer(1)
    }

    fn get_meta_data_for_anonymous_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> DataTypeMetaData {
        let data_type_ref = DataTypeRef::new_from_anonymous_value(self.get_data_type_id(), anonymous_value);

        data_type_ref.get_meta_data().to_owned()
    }

    fn get_meta_data_from_string(
        &self,
        string: &str,
    ) -> Result<DataTypeMetaData, String> {
        let parts: Vec<&str> = string.split(';').collect();

        if parts.len() < 1 {
            return Err("Invalid string data type format, expected string;{byte_count}".into());
        }

        let string_size = match parts[1].trim().parse::<u64>() {
            Ok(string_size) => string_size,
            Err(err) => {
                return Err(format!("Failed to parse string size: {}", err));
            }
        };

        Ok(DataTypeMetaData::SizedContainer(string_size))
    }
}
