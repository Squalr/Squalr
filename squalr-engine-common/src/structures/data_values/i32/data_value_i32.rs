use crate::structures::data_values::data_value::DataValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;

type PrimitiveType = i32;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DataValueI32 {
    pub value: PrimitiveType,
}

impl DataValue for DataValueI32 {
    fn get_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn get_value_string(&self) -> String {
        self.value.to_string()
    }

    fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) {
        self.value = PrimitiveType::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }

    fn clone_internal(&self) -> Box<dyn DataValue> {
        Box::new(self.clone())
    }

    fn serialize_internal(&self) -> Value {
        Value::String(self.get_value_string())
    }
}
