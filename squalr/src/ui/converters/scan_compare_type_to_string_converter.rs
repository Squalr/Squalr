use squalr_engine_api::structures::scanning::comparisons::{
    scan_compare_type::ScanCompareType, scan_compare_type_delta::ScanCompareTypeDelta, scan_compare_type_immediate::ScanCompareTypeImmediate,
    scan_compare_type_relative::ScanCompareTypeRelative,
};

pub struct ScanCompareTypeToStringConverter {}

impl ScanCompareTypeToStringConverter {
    pub fn convert_scan_compare_type_immediate_to_string(scan_compare_type_immediate: &ScanCompareTypeImmediate) -> &'static str {
        match scan_compare_type_immediate {
            ScanCompareTypeImmediate::Equal => "Equal to",
            ScanCompareTypeImmediate::NotEqual => "Not equal to",
            ScanCompareTypeImmediate::GreaterThan => "Greater than",
            ScanCompareTypeImmediate::GreaterThanOrEqual => "Greater than or equal to",
            ScanCompareTypeImmediate::LessThan => "Less than",
            ScanCompareTypeImmediate::LessThanOrEqual => "Less than or equal to",
        }
    }

    pub fn convert_scan_compare_type_delta_to_string(scan_compare_type_delta: &ScanCompareTypeDelta) -> &'static str {
        match scan_compare_type_delta {
            ScanCompareTypeDelta::IncreasedByX => "Increased by x",
            ScanCompareTypeDelta::DecreasedByX => "Decreased by x",
            ScanCompareTypeDelta::MultipliedByX => "Multiplied by x",
            ScanCompareTypeDelta::DividedByX => "Divided by x",
            ScanCompareTypeDelta::ModuloByX => "Modulo by x",
            ScanCompareTypeDelta::ShiftLeftByX => "Shifted left by x",
            ScanCompareTypeDelta::ShiftRightByX => "Shifted right by x",
            ScanCompareTypeDelta::LogicalAndByX => "Logical AND by x",
            ScanCompareTypeDelta::LogicalOrByX => "Logical OR by x",
            ScanCompareTypeDelta::LogicalXorByX => "Logical XOR by x",
        }
    }

    pub fn convert_scan_compare_type_relative_to_string(scan_compare_type_relative: &ScanCompareTypeRelative) -> &'static str {
        match scan_compare_type_relative {
            ScanCompareTypeRelative::Changed => "Changed",
            ScanCompareTypeRelative::Unchanged => "Unchanged",
            ScanCompareTypeRelative::Increased => "Increased",
            ScanCompareTypeRelative::Decreased => "Decreased",
        }
    }

    pub fn convert_scan_compare_type_to_string(scan_compare_type: &ScanCompareType) -> &'static str {
        match scan_compare_type {
            ScanCompareType::Immediate(im) => Self::convert_scan_compare_type_immediate_to_string(im),
            ScanCompareType::Delta(delta) => Self::convert_scan_compare_type_delta_to_string(delta),
            ScanCompareType::Relative(rel) => Self::convert_scan_compare_type_relative_to_string(rel),
        }
    }
}
