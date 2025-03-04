use squalr_engine_common::structures::{
    data_types::data_type::DataType,
    data_values::{anonymous_value::AnonymousValue, data_value::DataValue},
    scanning::scan_compare_type::ScanCompareType,
};

#[derive(Debug, Clone)]
pub struct ScanParameters {
    compare_type: ScanCompareType,
    compare_immediate: Option<AnonymousValue>,
}

impl ScanParameters {
    pub fn new() -> Self {
        Self {
            compare_type: ScanCompareType::Changed,
            compare_immediate: None,
        }
    }

    pub fn new_with_value(
        compare_type: ScanCompareType,
        value: Option<AnonymousValue>,
    ) -> Self {
        Self {
            compare_type,
            compare_immediate: value,
        }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.compare_type.clone()
    }

    pub fn deanonymize_type(
        &self,
        data_type: &Box<dyn DataType>,
    ) -> Box<dyn DataValue> {
        self.compare_immediate
            .as_ref()
            .and_then(|value| value.deanonymize_type(data_type).ok())
            .unwrap_or_else(|| panic!("Invalid type"))
    }

    pub fn get_compare_immediate(&self) -> Option<&AnonymousValue> {
        if self.is_immediate_comparison() {
            self.compare_immediate.as_ref()
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        if !self.is_immediate_comparison() {
            true
        } else {
            self.compare_immediate.is_some()
        }
    }

    pub fn clone(&self) -> Self {
        ScanParameters {
            compare_type: self.compare_type.clone(),
            compare_immediate: self.compare_immediate.clone(),
        }
    }
}
