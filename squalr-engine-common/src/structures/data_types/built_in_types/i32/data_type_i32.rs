use crate::structures::data_values::i32::data_value_i32::DataValueI32;
use crate::structures::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

type PrimitiveType = i32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeI32 {}

impl DataType for DataTypeI32 {
    fn get_name(&self) -> &str {
        &"i32"
    }

    fn get_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn get_endian(&self) -> Endian {
        Endian::Big
    }

    fn get_default_value(&self) -> Box<dyn DataValue> {
        Box::new(DataValueI32::default())
    }
}
