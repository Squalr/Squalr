use squalr_engine_api::structures::{
    data_values::{display_value::DisplayValue, display_value_type::DisplayValueType},
    scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    structs::container_type::ContainerType,
};

pub struct ElementScannerValueViewData {
    pub selected_scan_compare_type: ScanCompareType,
    pub current_scan_value: DisplayValue,
}

impl ElementScannerValueViewData {
    pub fn new() -> Self {
        Self {
            selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            current_scan_value: DisplayValue::new(String::new(), DisplayValueType::Decimal, ContainerType::None),
        }
    }
}
