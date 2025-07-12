use crate::structures::data_types::built_in_types::primitive_data_type::PrimitiveDataType;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

type PrimitiveType = i64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeI64be {}

impl DataTypeI64be {
    pub const DATA_TYPE_ID: &str = "i64be";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    fn to_vec(value: PrimitiveType) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }

    pub fn get_value_from_primitive(value: PrimitiveType) -> DataValue {
        let value_bytes = PrimitiveType::to_be_bytes(value);

        DataValue::new(DataTypeRef::new(Self::get_data_type_id()), value_bytes.to_vec())
    }
}

impl DataType for DataTypeI64be {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_unit_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn validate_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> bool {
        match PrimitiveDataType::deanonymize_primitive::<PrimitiveType>(anonymous_value_container, false) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn deanonymize_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> Result<DataValue, DataTypeError> {
        let value_bytes = PrimitiveDataType::deanonymize_primitive::<PrimitiveType>(anonymous_value_container, false)?;

        Ok(DataValue::new(DataTypeRef::new(Self::get_data_type_id()), value_bytes))
    }

    fn create_display_values(
        &self,
        value_bytes: &[u8],
    ) -> Result<DisplayValues, DataTypeError> {
        PrimitiveDataType::create_display_values(value_bytes, |value_bytes| {
            PrimitiveType::from_be_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
                value_bytes[4],
                value_bytes[5],
                value_bytes[6],
                value_bytes[7],
            ])
        })
    }

    fn get_supported_display_types(&self) -> Vec<DisplayValueType> {
        PrimitiveDataType::get_supported_display_types()
    }

    fn get_default_display_type(&self) -> DisplayValueType {
        DisplayValueType::Decimal
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn is_floating_point(&self) -> bool {
        false
    }

    fn is_signed(&self) -> bool {
        true
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        DataValue::new(data_type_ref, Self::to_vec(0))
    }
}
