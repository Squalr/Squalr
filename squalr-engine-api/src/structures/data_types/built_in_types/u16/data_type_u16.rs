use crate::structures::data_types::built_in_types::primitive_type::PrimitiveDataType;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

type PrimitiveType = u16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeU16 {}

impl DataTypeU16 {
    pub fn get_data_type_id() -> &'static str {
        &"u16"
    }

    fn to_vec(value: PrimitiveType) -> Vec<u8> {
        value.to_le_bytes().to_vec()
    }
}

impl DataType for DataTypeU16 {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Result<Vec<u8>, DataTypeError> {
        PrimitiveDataType::deanonymize_primitive::<PrimitiveType>(anonymous_value, false)
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
    ) -> Result<String, DataTypeError> {
        let expected = self.get_default_size_in_bytes() as usize;
        let actual = value_bytes.len();

        if actual == expected {
            Ok(PrimitiveType::from_le_bytes([value_bytes[0], value_bytes[1]]).to_string())
        } else {
            Err(DataTypeError::InvalidByteCount { expected, actual })
        }
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn is_discrete(&self) -> bool {
        true
    }

    fn get_default_value(
        &self,
        _data_type_meta_data: &DataTypeMetaData,
    ) -> DataValue {
        DataValue::new(self.get_data_type_id(), Self::to_vec(0))
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::None
    }
}
