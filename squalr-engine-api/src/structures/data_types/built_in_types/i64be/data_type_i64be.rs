use crate::structures::data_types::built_in_types::primitive_data_type::PrimitiveDataType;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::AnonymousValue;
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

        DataValue::new(DataTypeRef::new(Self::get_data_type_id(), DataTypeMetaData::Primitive()), value_bytes.to_vec())
    }
}

impl DataType for DataTypeI64be {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_default_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
    }

    fn validate_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> bool {
        let data_type_ref = DataTypeRef::new_from_anonymous_value(self.get_data_type_id(), anonymous_value);

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

        PrimitiveDataType::deanonymize_primitive::<PrimitiveType>(anonymous_value, data_type_ref, false)
    }

    fn create_display_values(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<DisplayValues, DataTypeError> {
        PrimitiveDataType::create_display_values(value_bytes, data_type_meta_data, || {
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

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn is_discrete(&self) -> bool {
        true
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        DataValue::new(data_type_ref, Self::to_vec(0))
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::Primitive()
    }

    fn get_meta_data_for_anonymous_value(
        &self,
        _anonymous_value: &AnonymousValue,
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
