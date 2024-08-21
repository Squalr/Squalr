use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_common::values::data_value::DataValue;

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
    
    pub fn get_compare_type(
        &self,
    ) -> ScanCompareType {
        self.compare_type.clone()
    }

    pub fn deanonymize_type(
        &self,
        data_type: &DataType,
    ) -> Option<DataValue> {
        if let Some(value) = &self.compare_immediate {
            return match value.deanonymize_type(data_type) {
                Ok(result) => Some(result),
                Err(_) => None,
            };
        }

        return None;
    }

    pub fn get_compare_immediate(
        &self,
    ) -> Option<&AnonymousValue> {
        if self.is_immediate_comparison() {
            return self.compare_immediate.as_ref();
        } else {
            return None;
        }
    }
    
    pub fn is_valid(
        &self,
    ) -> bool {
        if !self.is_immediate_comparison() {
            return true;
        } else {
            return self.compare_immediate.is_some();
        }
    }

    pub fn is_relative_delta_comparison(
        &self,
    ) -> bool {
        return match self.compare_type {
            | ScanCompareType::IncreasedByX
            | ScanCompareType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn is_relative_comparison(
        &self,
    ) -> bool {
        return match self.compare_type {
            ScanCompareType::Changed
            | ScanCompareType::Unchanged
            | ScanCompareType::Increased
            | ScanCompareType::Decreased => true,
            _ => false,
        };
    }

    pub fn is_immediate_comparison(
        &self,
    ) -> bool {
        return match self.compare_type {
            ScanCompareType::Equal
            | ScanCompareType::NotEqual
            | ScanCompareType::GreaterThan
            | ScanCompareType::GreaterThanOrEqual
            | ScanCompareType::LessThan
            | ScanCompareType::LessThanOrEqual
            | ScanCompareType::IncreasedByX
            | ScanCompareType::DecreasedByX => true,
            _ => false,
        };
    }

    pub fn clone(
        &self,
    ) -> Self {
        ScanParameters {
            compare_type: self.compare_type.clone(),
            compare_immediate: self.compare_immediate.clone(),
        }
    }

    pub fn conflicts_with(
        &self,
        other: &ScanParameters
    ) -> bool {
        if self.compare_type == other.compare_type {
            return true;
        }

        if !self.is_immediate_comparison() && !other.is_immediate_comparison() {
            return true;
        }

        if self.is_immediate_comparison() && other.is_immediate_comparison() {
            if (matches!(self.compare_type, ScanCompareType::LessThan | ScanCompareType::LessThanOrEqual | ScanCompareType::NotEqual)
                && matches!(other.compare_type, ScanCompareType::GreaterThan | ScanCompareType::GreaterThanOrEqual | ScanCompareType::NotEqual))
                || (matches!(self.compare_type, ScanCompareType::GreaterThan | ScanCompareType::GreaterThanOrEqual | ScanCompareType::NotEqual)
                    && matches!(other.compare_type, ScanCompareType::LessThan | ScanCompareType::LessThanOrEqual | ScanCompareType::NotEqual))
            {
                return true;
            }
        }

        return false;
    }
}
