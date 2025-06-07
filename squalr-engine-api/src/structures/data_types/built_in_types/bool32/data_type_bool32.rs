use crate::structures::data_types::built_in_types::primitive_data_type::PrimitiveDataType;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::AnonymousValueContainer;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

// For a bool32 the underlying primitive is a u32.
type ExposedType = bool;
type PrimitiveType = u32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeBool32 {}

impl DataTypeBool32 {
    pub const DATA_TYPE_ID: &str = "bool32";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    fn to_vec(value: ExposedType) -> Vec<u8> {
        (value as PrimitiveType).to_le_bytes().to_vec()
    }

    pub fn get_value_from_primitive(value: ExposedType) -> DataValue {
        let value_bytes = PrimitiveType::to_le_bytes(value as PrimitiveType);

        DataValue::new(DataTypeRef::new(Self::get_data_type_id(), DataTypeMetaData::Primitive()), value_bytes.to_vec())
    }
}

impl DataType for DataTypeBool32 {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        "bool"
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn validate_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> bool {
        match PrimitiveDataType::deanonymize_bool::<PrimitiveType>(anonymous_value_container, false) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn deanonymize_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> Result<DataValue, DataTypeError> {
        Ok(DataValue::new(
            DataTypeRef::new(Self::get_data_type_id(), self.get_meta_data_for_anonymous_value(anonymous_value_container)),
            PrimitiveDataType::deanonymize_bool::<PrimitiveType>(anonymous_value_container, false)?,
        ))
    }

    fn array_merge(
        &self,
        data_values: Vec<DataValue>,
    ) -> Result<DataValue, DataTypeError> {
        PrimitiveDataType::array_merge(data_values)
    }

    fn create_display_values(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<DisplayValues, DataTypeError> {
        PrimitiveDataType::create_display_values_bool(value_bytes, self.get_default_size_in_bytes(), data_type_meta_data)
    }

    fn get_supported_display_types(&self) -> Vec<DisplayValueType> {
        vec![
            DisplayValueType::Bool(ContainerType::None),
            DisplayValueType::Bool(ContainerType::Array),
        ]
    }

    fn get_default_display_type(&self) -> DisplayValueType {
        DisplayValueType::Bool(ContainerType::None)
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn is_floating_point(&self) -> bool {
        false
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        DataValue::new(data_type_ref, Self::to_vec(false))
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::Primitive()
    }

    fn get_meta_data_for_anonymous_value(
        &self,
        _anonymous_value_container: &AnonymousValueContainer,
    ) -> DataTypeMetaData {
        DataTypeMetaData::Primitive()
    }

    fn get_meta_data_from_string(
        &self,
        _string: &str,
    ) -> Result<DataTypeMetaData, String> {
        Ok(DataTypeMetaData::Primitive())
    }
}
