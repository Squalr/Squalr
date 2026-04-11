use crate::Arm64InstructionSet;
use squalr_engine_api::{
    impl_instruction_data_type_comparison_stubs,
    plugins::instruction_set::{InstructionSet, anonymize_instruction_bytes, deanonymize_instruction_value},
    structures::{
        data_types::{data_type::DataType, data_type_error::DataTypeError, data_type_ref::DataTypeRef},
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue},
        memory::endian::Endian,
    },
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DataTypeIArm64 {
    instruction_set: Arc<dyn InstructionSet>,
}

impl DataTypeIArm64 {
    pub const DATA_TYPE_ID: &str = "i_arm64";

    pub fn new() -> Self {
        Self {
            instruction_set: Arc::new(Arm64InstructionSet::new()),
        }
    }
}

impl Default for DataTypeIArm64 {
    fn default() -> Self {
        Self::new()
    }
}

impl DataType for DataTypeIArm64 {
    fn get_data_type_id(&self) -> &str {
        Self::DATA_TYPE_ID
    }

    fn get_icon_id(&self) -> &str {
        "cpu_instruction"
    }

    fn get_unit_size_in_bytes(&self) -> u64 {
        1
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
        deanonymize_instruction_value(Self::DATA_TYPE_ID, self.instruction_set.as_ref(), anonymous_value_string)
    }

    fn anonymize_value_bytes(
        &self,
        value_bytes: &[u8],
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError> {
        anonymize_instruction_bytes(self.instruction_set.as_ref(), Self::DATA_TYPE_ID, value_bytes, anonymous_value_string_format)
    }

    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
        vec![
            AnonymousValueStringFormat::String,
            AnonymousValueStringFormat::Hexadecimal,
        ]
    }

    fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
        AnonymousValueStringFormat::String
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn is_floating_point(&self) -> bool {
        false
    }

    fn is_signed(&self) -> bool {
        false
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        DataValue::new(data_type_ref, Vec::new())
    }
}

impl_instruction_data_type_comparison_stubs!(DataTypeIArm64);
