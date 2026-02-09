use crate::structures::data_types::built_in_types::primitive_data_type_numeric::PrimitiveDataTypeNumeric;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
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

    pub fn get_icon_id() -> &'static str {
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
        Self::get_icon_id()
    }

    fn get_unit_size_in_bytes(&self) -> u64 {
        size_of::<PrimitiveType>() as u64
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
        let value_bytes = PrimitiveDataTypeNumeric::deanonymize::<PrimitiveType>(anonymous_value_string, true)?;

        Ok(DataValue::new(DataTypeRef::new(Self::get_data_type_id()), value_bytes))
    }

    fn anonymize_value_bytes(
        &self,
        value_bytes: &[u8],
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        let anonymous_value_string = PrimitiveDataTypeNumeric::anonymize(
            value_bytes,
            |value_bytes| {
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
            },
            anonymous_value_string_format,
        )?;

        Ok(anonymous_value_string)
    }

    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
        PrimitiveDataTypeNumeric::get_supported_anonymous_value_string_formats()
    }

    fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
        AnonymousValueStringFormat::Decimal
    }

    fn get_endian(&self) -> Endian {
        Endian::Big
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
