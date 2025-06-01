use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::{AnonymousValue, AnonymousValueContainer};
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

/// Represents the 'data type ref' data type, ie a data type that references another data type.
/// In other words, this is a data type that contains a fixed, known `String`, used to construct a `DataTypeRef`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeRefDataType {}

impl DataTypeRefDataType {
    pub const DATA_TYPE_ID: &str = "data_type_ref";

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

    pub fn resolve_data_type_reference(data_type_meta_data: &DataTypeMetaData) -> DataTypeRef {
        match data_type_meta_data {
            DataTypeMetaData::FixedString(data_type_ref_id) => DataTypeRef::new(data_type_ref_id, DataTypeMetaData::None),
            _ => DataTypeRef::new("".into(), DataTypeMetaData::None),
        }
    }
}

impl DataType for DataTypeRefDataType {
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
            return Err(DataTypeError::InvalidDataTypeRef {
                data_type_ref: data_type_ref.get_data_type_id().to_string(),
            });
        }

        match data_type_ref.get_meta_data() {
            DataTypeMetaData::FixedString(_string) => match anonymous_value.get_value() {
                AnonymousValueContainer::String(value_string) => {
                    let string_bytes = value_string.as_bytes().to_vec();

                    Ok(DataValue::new(data_type_ref, string_bytes))
                }
                _ => Err(DataTypeError::InvalidMetaData),
            },
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn create_display_values(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<DisplayValues, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        match data_type_meta_data {
            DataTypeMetaData::FixedString(string) => Ok(DisplayValues::new(
                vec![DisplayValue::new(
                    DisplayValueType::DataTypeRef(ContainerType::None),
                    string.into(),
                )],
                DisplayValueType::DataTypeRef(ContainerType::None),
            )),
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn get_supported_display_types(&self) -> Vec<DisplayValueType> {
        vec![
            DisplayValueType::DataTypeRef(ContainerType::None),
            DisplayValueType::DataTypeRef(ContainerType::Array),
        ]
    }

    fn get_default_display_type(&self) -> DisplayValueType {
        DisplayValueType::DataTypeRef(ContainerType::None)
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
        DataTypeMetaData::FixedString(DataTypeRefDataType::get_data_type_id().to_string())
    }

    fn get_meta_data_for_anonymous_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> DataTypeMetaData {
        match anonymous_value.get_value() {
            AnonymousValueContainer::String(string) => DataTypeMetaData::FixedString(string.into()),

            // These anonymous container types are not supported.
            AnonymousValueContainer::BinaryValue(_) => DataTypeMetaData::FixedString("".into()),
            AnonymousValueContainer::HexadecimalValue(_) => DataTypeMetaData::FixedString("".into()),
        }
    }

    fn get_meta_data_from_string(
        &self,
        string: &str,
    ) -> Result<DataTypeMetaData, String> {
        Ok(DataTypeMetaData::FixedString(string.to_string()))
    }
}
