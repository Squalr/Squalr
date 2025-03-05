use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

type PrimitiveType = i32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeI32 {}

impl DataTypeI32 {
    fn to_vec(value: i32) -> Vec<u8> {
        value.to_le_bytes().to_vec()
    }
}

impl DataType for DataTypeI32 {
    fn get_id(&self) -> &str {
        &"i32"
    }

    fn get_icon_id(&self) -> &str {
        &"i32"
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Vec<u8> {
        let value_string = anonymous_value.to_string();

        match value_string.parse::<i32>() {
            Ok(value) => Self::to_vec(value),
            Err(_) => vec![],
        }
    }

    fn get_endian(&self) -> Endian {
        Endian::Big
    }

    fn get_default_value(&self) -> DataValue {
        DataValue::new(self.get_ref(), Self::to_vec(0))
    }
}
