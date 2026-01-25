use crate::structures::data_types::built_in_types::primitive_data_type::PrimitiveDataType;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value_interpretation_format::DataValueInterpretationFormat;
use crate::structures::data_values::data_value_interpreter::DataValueInterpreter;
use crate::structures::data_values::data_value_interpreters::DataValueInterpreters;
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

    pub fn get_value_from_primitive(string_byte: u8) -> DataValue {
        DataValue::new(DataTypeRef::new(Self::get_data_type_id()), vec![string_byte])
    }

    pub fn get_value_from_primitive_array(string_bytes: Vec<u8>) -> DataValue {
        DataValue::new(DataTypeRef::new(Self::get_data_type_id()), string_bytes)
    }

    pub fn get_value_from_primitive_string(string: &str) -> DataValue {
        Self::get_value_from_primitive_array(string.as_bytes().to_vec())
    }
}

impl DataType for DataTypeStringUtf8 {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        "string"
    }

    fn get_unit_size_in_bytes(&self) -> u64 {
        1
    }

    fn validate_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> bool {
        match anonymous_value_container {
            AnonymousValueContainer::BinaryValue(value_string) => !value_string.is_empty(),
            AnonymousValueContainer::HexadecimalValue(value_string) => !value_string.is_empty(),
            AnonymousValueContainer::String(value_string) => !value_string.is_empty(),
        }
    }

    fn deanonymize_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> Result<DataValue, DataTypeError> {
        let data_type_ref = DataTypeRef::new(Self::get_data_type_id());
        let decoded_bytes = PrimitiveDataType::decode_string(anonymous_value_container, |value_string| value_string.as_bytes().to_vec())?;

        Ok(DataValue::new(data_type_ref, decoded_bytes))
    }

    fn create_data_value_interpreters(
        &self,
        value_bytes: &[u8],
    ) -> Result<DataValueInterpreters, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        let decoded_string = std::str::from_utf8(value_bytes)
            .map_err(|_err| DataTypeError::DecodingError)?
            .to_string();

        Ok(DataValueInterpreters::new(
            vec![DataValueInterpreter::new(
                decoded_string,
                DataValueInterpretationFormat::String,
                ContainerType::None,
            )],
            DataValueInterpretationFormat::String,
        ))
    }

    fn get_supported_display_types(&self) -> Vec<DataValueInterpretationFormat> {
        vec![DataValueInterpretationFormat::String]
    }

    fn get_default_display_type(&self) -> DataValueInterpretationFormat {
        DataValueInterpretationFormat::String
    }

    fn is_floating_point(&self) -> bool {
        false
    }

    fn is_signed(&self) -> bool {
        false
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
}
