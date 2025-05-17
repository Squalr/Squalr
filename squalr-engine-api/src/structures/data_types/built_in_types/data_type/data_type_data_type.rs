use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::{AnonymousValue, AnonymousValueContainer};
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

/// Represents the 'data type data type', ie a data type that references another data type.
/// In other words, this is a data type that contains a fixed, known `String`, used to construct a `DataTypeRef`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeDataType {}

impl DataTypeDataType {
    pub const DATA_TYPE_ID: &str = "data_type";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_value_from_primitive(str: &str) -> DataValue {
        let value_bytes = str.as_bytes();
        DataValue::new(
            DataTypeRef::new(Self::get_data_type_id(), DataTypeMetaData::FixedString(str.to_string())),
            value_bytes.to_vec(),
        )
    }
}

impl DataType for DataTypeDataType {
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

        // Validating a UTF string really just boils down to "can we parse the anonymous value as a string".
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

        match data_type_ref.get_meta_data() {
            DataTypeMetaData::FixedString(_string) => match anonymous_value.get_value() {
                AnonymousValueContainer::StringValue(value_string, _is_value_hex) => {
                    let string_bytes = value_string.as_bytes().to_vec();

                    Ok(DataValue::new(data_type_ref, string_bytes))
                }
                AnonymousValueContainer::ByteArray(value_bytes) => Ok(DataValue::new(data_type_ref, value_bytes.clone())),
            },
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<String, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        match data_type_meta_data {
            DataTypeMetaData::FixedString(string) => Ok(string.into()),
            _ => Err(DataTypeError::InvalidMetaData),
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
        DataValue::new(data_type_ref.clone(), vec![])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::FixedString(DataTypeDataType::get_data_type_id().to_string())
    }

    fn get_meta_data_for_anonymous_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> DataTypeMetaData {
        match anonymous_value.get_value() {
            AnonymousValueContainer::ByteArray(byte_array) => DataTypeMetaData::FixedString(std::str::from_utf8(byte_array).unwrap_or_default().to_string()),
            AnonymousValueContainer::StringValue(string, _is_hex) => DataTypeMetaData::FixedString(string.into()),
        }
    }

    fn get_meta_data_from_string(
        &self,
        string: &str,
    ) -> Result<DataTypeMetaData, String> {
        Ok(DataTypeMetaData::FixedString(string.to_string()))
    }
}
