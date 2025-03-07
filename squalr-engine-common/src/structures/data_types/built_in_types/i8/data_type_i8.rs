use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

type PrimitiveType = i8;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeI8 {}

impl DataTypeI8 {
    pub fn get_id() -> &'static str {
        &"i8"
    }

    fn to_vec(value: PrimitiveType) -> Vec<u8> {
        value.to_le_bytes().to_vec()
    }
}

impl DataType for DataTypeI8 {
    fn get_id(&self) -> &str {
        Self::get_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_id()
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Vec<u8> {
        let value_string = anonymous_value.to_string();

        match value_string.parse::<PrimitiveType>() {
            Ok(value) => Self::to_vec(value),
            Err(_) => vec![],
        }
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
    ) -> Option<String> {
        if value_bytes.len() == self.get_default_size_in_bytes() as usize {
            Some(PrimitiveType::from_le_bytes([value_bytes[0]]).to_string())
        } else {
            None
        }
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn get_default_value(&self) -> DataValue {
        DataValue::new(self.get_ref(), Self::to_vec(0))
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::None
    }
}
