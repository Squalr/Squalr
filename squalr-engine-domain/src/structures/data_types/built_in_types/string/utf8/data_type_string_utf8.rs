use crate::structures::data_types::built_in_types::primitive_data_type_string::PrimitiveDataTypeString;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
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

    fn validate_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        match self.deanonymize_value_string(anonymous_value_string) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn deanonymize_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, DataTypeError> {
        let data_type_ref = DataTypeRef::new(Self::get_data_type_id());
        let decoded_bytes = PrimitiveDataTypeString::deanonymize_string(anonymous_value_string, |value_string| value_string.as_bytes().to_vec())?;

        Ok(DataValue::new(data_type_ref, decoded_bytes))
    }

    fn anonymize_value_bytes(
        &self,
        value_bytes: &[u8],
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        if anonymous_value_string_format != AnonymousValueStringFormat::String {
            return Err(DataTypeError::ParseError("Unsupported data value format".to_string()));
        }

        let decoded_string = String::from_utf8_lossy(value_bytes).to_string();

        Ok(AnonymousValueString::new(
            decoded_string,
            AnonymousValueStringFormat::String,
            crate::structures::data_values::container_type::ContainerType::None,
        ))
    }

    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
        PrimitiveDataTypeString::get_supported_anonymous_value_string_formats()
    }

    fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
        AnonymousValueStringFormat::String
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

#[cfg(test)]
mod tests {
    use super::DataTypeStringUtf8;
    use crate::structures::data_types::data_type::DataType;
    use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;

    #[test]
    fn anonymize_value_bytes_returns_utf8_string() {
        let data_type = DataTypeStringUtf8 {};
        let value_bytes = b"PlayerHealth";
        let anonymous_value_string = data_type
            .anonymize_value_bytes(value_bytes, AnonymousValueStringFormat::String)
            .unwrap_or_else(|error| panic!("Expected UTF-8 anonymization to succeed: {}", error));

        assert_eq!(anonymous_value_string.get_anonymous_value_string(), "PlayerHealth");
    }

    #[test]
    fn anonymize_value_bytes_with_invalid_utf8_replaces_invalid_sequences() {
        let data_type = DataTypeStringUtf8 {};
        let value_bytes = [0xF0, 0x80, 0x80, 0x80];
        let anonymous_value_string = data_type
            .anonymize_value_bytes(&value_bytes, AnonymousValueStringFormat::String)
            .unwrap_or_else(|error| panic!("Expected UTF-8 anonymization to tolerate invalid bytes: {}", error));

        assert!(!anonymous_value_string.get_anonymous_value_string().is_empty());
    }
}
