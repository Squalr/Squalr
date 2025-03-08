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
    ) -> Vec<u8> {
        let value_string = anonymous_value.to_string();

        value_string
            .split_whitespace()
            .filter_map(|hex_str| u8::from_str_radix(hex_str, 16).ok())
            .collect()
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
    ) -> Option<String> {
        if !value_bytes.is_empty() {
            Some(
                value_bytes
                    .iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(" "),
            )
        } else {
            None
        }
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn get_default_value(&self) -> DataValue {
        DataValue::new(self.get_ref(), vec![])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::SizedContainer(1)
    }
}
