use crate::structures::data_types::built_in_types::bool8::data_type_bool8::DataTypeBool8;
use crate::structures::data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8;
use crate::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scan_results::scan_result_base::ScanResultBase;
use crate::structures::scan_results::scan_result_valued::ScanResultValued;
use crate::structures::structs::valued_struct::ValuedStruct;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScanResult {
    valued_result: ScanResultValued,
    module: String,
    module_offset: u64,
    recently_read_value: Option<DataValue>,
    is_frozen: bool,
}

impl ScanResult {
    pub const PROPERTY_NAME_VALUE: &str = "value";
    pub const PROPERTY_NAME_IS_FROZEN: &str = "is_frozen";
    pub const PROPERTY_NAME_ADDRESS: &str = "address";
    pub const PROPERTY_NAME_MODULE: &str = "module";
    pub const PROPERTY_NAME_MODULE_OFFSET: &str = "module_offset";

    pub fn new(
        valued_result: ScanResultValued,
        module: String,
        module_offset: u64,
        recently_read_value: Option<DataValue>,
        is_frozen: bool,
    ) -> Self {
        Self {
            valued_result,
            module,
            module_offset,
            recently_read_value,
            is_frozen,
        }
    }

    pub fn as_property_struct(&self) -> ValuedStruct {
        // The current value if available, otherwise fall back to a read only default string.
        let field_value = match self.valued_result.get_current_value() {
            Some(current_value) => current_value
                .clone()
                .to_named_valued_struct_field(Self::PROPERTY_NAME_VALUE.to_string(), false),
            None => DataTypeStringUtf8::get_value_from_primitive('?' as u8).to_named_valued_struct_field(Self::PROPERTY_NAME_VALUE.to_string(), true),
        };
        let field_is_frozen =
            DataTypeBool8::get_value_from_primitive(self.is_frozen).to_named_valued_struct_field(Self::PROPERTY_NAME_IS_FROZEN.to_string(), false);
        let field_address =
            DataTypeU64::get_value_from_primitive(self.valued_result.get_address()).to_named_valued_struct_field(Self::PROPERTY_NAME_ADDRESS.to_string(), true);
        let field_module = DataTypeStringUtf8::get_value_from_primitive_array(self.module.as_bytes().to_vec())
            .to_named_valued_struct_field(Self::PROPERTY_NAME_MODULE.to_string(), true);
        let field_module_offset =
            DataTypeU64::get_value_from_primitive(self.module_offset).to_named_valued_struct_field(Self::PROPERTY_NAME_MODULE_OFFSET.to_string(), true);

        ValuedStruct::new_anonymous(vec![
            field_value,
            field_is_frozen,
            field_address,
            field_module,
            field_module_offset,
        ])
    }

    pub fn get_valued_result(&self) -> &ScanResultValued {
        &self.valued_result
    }

    pub fn get_base_result(&self) -> &ScanResultBase {
        self.valued_result.get_base_result()
    }

    pub fn get_address(&self) -> u64 {
        self.valued_result.get_address()
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.valued_result.get_data_type()
    }

    pub fn is_module(&self) -> bool {
        !self.module.is_empty()
    }

    pub fn get_module(&self) -> &str {
        &self.module
    }

    pub fn get_module_offset(&self) -> u64 {
        self.module_offset
    }

    pub fn get_recently_read_value(&self) -> &Option<DataValue> {
        &self.recently_read_value
    }

    pub fn get_current_value(&self) -> &Option<DataValue> {
        &self.valued_result.get_current_value()
    }

    pub fn get_previous_value(&self) -> &Option<DataValue> {
        &self.valued_result.get_previous_value()
    }

    pub fn get_is_frozen(&self) -> bool {
        self.is_frozen
    }
}

impl fmt::Debug for ScanResult {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        if self.module.is_empty() {
            write!(formatter, "ScanResult {{ address: 0x{:X} }}", self.get_address())
        } else {
            write!(
                formatter,
                "ScanResult {{ module: {} }}, {{ offset: 0x{:X} }}, ",
                self.module, self.module_offset
            )
        }
    }
}
