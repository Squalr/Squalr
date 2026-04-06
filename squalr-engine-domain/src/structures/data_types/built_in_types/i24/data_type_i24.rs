use crate::structures::data_types::built_in_types::primitive_data_type_24_bit::PrimitiveDataType24Bit;
use crate::structures::data_types::data_type::DataType;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::endian::Endian;
use serde::{Deserialize, Serialize};

type PrimitiveType = i32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeI24;

impl DataTypeI24 {
    pub const DATA_TYPE_ID: &str = "i24";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_icon_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_value_from_primitive(value: PrimitiveType) -> DataValue {
        DataValue::new(
            DataTypeRef::new(Self::get_data_type_id()),
            PrimitiveDataType24Bit::get_signed_value_bytes(value, Endian::Little),
        )
    }
}

impl DataType for DataTypeI24 {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_icon_id()
    }

    fn get_unit_size_in_bytes(&self) -> u64 {
        3
    }

    fn validate_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        self.deanonymize_value_string(anonymous_value_string).is_ok()
    }

    fn deanonymize_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, DataTypeError> {
        PrimitiveDataType24Bit::deanonymize_signed(anonymous_value_string, Endian::Little)
            .map(|value_bytes| DataValue::new(DataTypeRef::new(Self::get_data_type_id()), value_bytes))
    }

    fn anonymize_value_bytes(
        &self,
        value_bytes: &[u8],
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        PrimitiveDataType24Bit::anonymize_signed(value_bytes, Endian::Little, anonymous_value_string_format)
    }

    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
        PrimitiveDataType24Bit::get_supported_anonymous_value_string_formats()
    }

    fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
        AnonymousValueStringFormat::Decimal
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
        DataValue::new(data_type_ref, PrimitiveDataType24Bit::get_signed_value_bytes(0, Endian::Little))
    }
}