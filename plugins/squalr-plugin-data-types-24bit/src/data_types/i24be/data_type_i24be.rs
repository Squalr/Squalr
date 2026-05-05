use crate::data_types::primitive_data_type_24_bit::PrimitiveDataType24Bit;
use squalr_engine_api::structures::data_types::data_type::DataType;
use squalr_engine_api::structures::data_types::data_type_error::DataTypeError;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_types::data_type_scan_preference::DataTypeScanPreference;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::memory::endian::Endian;

type PrimitiveType = i32;

#[derive(Clone, Debug)]
pub struct DataTypeI24be;

impl DataTypeI24be {
    pub const DATA_TYPE_ID: &str = "i24be";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_icon_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_value_from_primitive(value: PrimitiveType) -> DataValue {
        DataValue::new(
            DataTypeRef::new(Self::get_data_type_id()),
            PrimitiveDataType24Bit::get_signed_value_bytes(value, Endian::Big),
        )
    }
}

impl DataType for DataTypeI24be {
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
        PrimitiveDataType24Bit::deanonymize_signed(anonymous_value_string, Endian::Big)
            .map(|value_bytes| DataValue::new(DataTypeRef::new(Self::get_data_type_id()), value_bytes))
    }

    fn anonymize_value_bytes(
        &self,
        value_bytes: &[u8],
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        PrimitiveDataType24Bit::anonymize_signed(value_bytes, Endian::Big, anonymous_value_string_format)
    }

    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
        PrimitiveDataType24Bit::get_supported_anonymous_value_string_formats()
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

    fn supports_scalar_integer_values(&self) -> bool {
        true
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        DataValue::new(data_type_ref, PrimitiveDataType24Bit::get_signed_value_bytes(0, Endian::Big))
    }

    fn get_scan_preference(&self) -> DataTypeScanPreference {
        DataTypeScanPreference::PreferTypeScanner
    }
}
