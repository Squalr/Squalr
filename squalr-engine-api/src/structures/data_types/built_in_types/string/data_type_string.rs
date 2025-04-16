use crate::structures::data_types::built_in_types::string::string_encodings::StringEncoding;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::{AnonymousValue, AnonymousValueContainer};
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeString {}

impl DataTypeString {
    pub const DATA_TYPE_ID: &str = "string";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }
}

impl DataType for DataTypeString {
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
        let data_type_ref = DataTypeRef::new(self.get_data_type_id(), DataTypeMetaData::EncodedString(0, StringEncoding::Utf8));

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
        /*
        match anonymous_value.get_value() {
            AnonymousValueContainer::StringValue(value_string, is_value_hex) => Ok(vec![]),
            AnonymousValueContainer::ByteArray(value_bytes) => Ok(value_bytes.clone()),
        }*/
        Err(DataTypeError::NoBytes)
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
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
            DataTypeMetaData::EncodedString(size, encoding) => *size as usize,
            _ => {
                log::error!("Invalid metadata provided to byte array data type.");
                0usize
            }
        };

        DataValue::new(data_type_ref.clone(), vec![0u8; array_size])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::EncodedString(1, StringEncoding::Utf8)
    }
}
