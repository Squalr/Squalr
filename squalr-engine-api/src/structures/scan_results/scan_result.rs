use crate::structures::data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8;
use crate::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scan_results::scan_result_base::ScanResultBase;
use crate::structures::scan_results::scan_result_valued::ScanResultValued;
use crate::structures::structs::valued_struct::ValuedStruct;
use crate::structures::{data_types::built_in_types::bool8::data_type_bool8::DataTypeBool8, data_values::anonymous_value_string::AnonymousValueString};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct ScanResult {
    valued_result: ScanResultValued,
    module: String,
    module_offset: u64,
    recently_read_value: Option<DataValue>,
    recently_read_display_values: Vec<AnonymousValueString>,
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
        recently_read_display_values: Vec<AnonymousValueString>,
        is_frozen: bool,
    ) -> Self {
        Self {
            valued_result,
            module,
            module_offset,
            recently_read_value,
            recently_read_display_values,
            is_frozen,
        }
    }

    pub fn as_valued_struct(&self) -> ValuedStruct {
        // The current value if available, otherwise fall back to a read only default string.
        let field_value = match self
            .recently_read_value
            .as_ref()
            .or_else(|| self.valued_result.get_current_value().as_ref())
        {
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

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.valued_result.get_data_type_ref()
    }

    pub fn get_icon_id(&self) -> &str {
        &self.valued_result.get_icon_id()
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

    pub fn get_recently_read_display_values(&self) -> &Vec<AnonymousValueString> {
        &self.recently_read_display_values
    }

    pub fn get_recently_read_display_value(
        &self,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Option<&AnonymousValueString> {
        for recently_read_display_value in &self.recently_read_display_values {
            if recently_read_display_value.get_anonymous_value_string_format() == anonymous_value_string_format {
                return Some(recently_read_display_value);
            }
        }

        None
    }

    pub fn get_current_value(&self) -> &Option<DataValue> {
        &self.valued_result.get_current_value()
    }

    pub fn get_current_display_values(&self) -> &Vec<AnonymousValueString> {
        &self.valued_result.get_current_display_values()
    }

    pub fn get_current_display_value(
        &self,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Option<&AnonymousValueString> {
        self.valued_result
            .get_current_display_value(anonymous_value_string_format)
    }

    pub fn get_previous_value(&self) -> &Option<DataValue> {
        &self.valued_result.get_previous_value()
    }

    pub fn get_previous_display_values(&self) -> &Vec<AnonymousValueString> {
        &self.valued_result.get_previous_display_values()
    }

    pub fn get_previous_display_value(
        &self,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Option<&AnonymousValueString> {
        self.valued_result
            .get_previous_display_value(anonymous_value_string_format)
    }

    pub fn get_is_frozen(&self) -> bool {
        self.is_frozen
    }

    pub fn set_is_frozen_client_only(
        &mut self,
        is_frozen: bool,
    ) {
        self.is_frozen = is_frozen;
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

#[cfg(test)]
mod tests {
    use super::ScanResult;
    use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::data_values::data_value::DataValue;
    use crate::structures::scan_results::scan_result_ref::ScanResultRef;
    use crate::structures::scan_results::scan_result_valued::ScanResultValued;

    fn create_scan_result() -> ScanResult {
        let scan_result_valued = ScanResultValued::new(
            0x1000,
            DataTypeRef::new("u8"),
            String::new(),
            Some(DataTypeU8::get_value_from_primitive(42)),
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(1),
        );

        ScanResult::new(scan_result_valued, String::from("module"), 0x20, None, Vec::new(), true)
    }

    #[test]
    fn as_valued_struct_allows_writing_value_and_is_frozen_fields() {
        let scan_result = create_scan_result();
        let valued_struct = scan_result.as_valued_struct();

        let value_field = valued_struct
            .get_field(ScanResult::PROPERTY_NAME_VALUE)
            .expect("Expected value field.");
        let is_frozen_field = valued_struct
            .get_field(ScanResult::PROPERTY_NAME_IS_FROZEN)
            .expect("Expected is_frozen field.");
        let address_field = valued_struct
            .get_field(ScanResult::PROPERTY_NAME_ADDRESS)
            .expect("Expected address field.");
        let module_field = valued_struct
            .get_field(ScanResult::PROPERTY_NAME_MODULE)
            .expect("Expected module field.");
        let module_offset_field = valued_struct
            .get_field(ScanResult::PROPERTY_NAME_MODULE_OFFSET)
            .expect("Expected module_offset field.");

        assert!(!value_field.get_is_read_only());
        assert!(!is_frozen_field.get_is_read_only());
        assert!(address_field.get_is_read_only());
        assert!(module_field.get_is_read_only());
        assert!(module_offset_field.get_is_read_only());
    }

    #[test]
    fn as_valued_struct_prefers_recently_read_value_for_value_field() {
        let scan_result_valued = ScanResultValued::new(
            0x1000,
            DataTypeRef::new("u8"),
            String::new(),
            Some(DataTypeU8::get_value_from_primitive(10)),
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(1),
        );
        let scan_result = ScanResult::new(
            scan_result_valued,
            String::from("module"),
            0x20,
            Some(DataTypeU8::get_value_from_primitive(25)),
            Vec::new(),
            false,
        );
        let valued_struct = scan_result.as_valued_struct();
        let value_field = valued_struct
            .get_field(ScanResult::PROPERTY_NAME_VALUE)
            .expect("Expected value field.");
        let value_data_value = value_field
            .get_data_value()
            .expect("Expected value data value.");
        let expected_value: DataValue = DataTypeU8::get_value_from_primitive(25);

        assert_eq!(value_data_value.get_value_bytes(), expected_value.get_value_bytes());
    }
}
