use squalr_engine_common::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{anonymous_value::AnonymousValue, data_value::DataValue},
    scanning::scan_compare_type::ScanCompareType,
};

#[derive(Debug, Clone)]
pub struct ScanParameters {
    compare_type: ScanCompareType,
    compare_immediate: Option<AnonymousValue>,
}

impl ScanParameters {
    pub fn new(
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

    /// Tries to deanonymizes the scan arg into a usable `DataValue` based on the provided `DataType`.
    pub fn deanonymize_immediate(
        &self,
        data_type: &DataTypeRef,
    ) -> Option<DataValue> {
        match &self.compare_immediate {
            Some(anonymous_value) => data_type.deanonymize_value(&anonymous_value),
            None => None,
        }
    }

    pub fn get_compare_immediate(&self) -> Option<&AnonymousValue> {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) => self.compare_immediate.as_ref(),
            ScanCompareType::Relative(_) => None,
            ScanCompareType::Delta(_) => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) | ScanCompareType::Delta(_) => self.compare_immediate.is_some(),
            ScanCompareType::Relative(_) => true,
        }
    }

    pub fn clone(&self) -> Self {
        ScanParameters {
            compare_type: self.compare_type.clone(),
            compare_immediate: self.compare_immediate.clone(),
        }
    }
}
