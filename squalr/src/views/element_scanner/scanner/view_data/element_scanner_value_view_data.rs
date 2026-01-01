use squalr_engine_api::structures::{
    data_values::{display_value::DisplayValue, display_value_type::DisplayValueType},
    scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    structs::container_type::ContainerType,
};

#[derive(Clone)]
pub struct ElementScannerValueViewData {
    pub selected_scan_compare_type: ScanCompareType,
    pub current_scan_value: DisplayValue,
    pub menu_id: String,
}

impl ElementScannerValueViewData {
    pub fn new(menu_id: String) -> Self {
        Self {
            selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            current_scan_value: DisplayValue::new(String::new(), DisplayValueType::Decimal, ContainerType::None),
            menu_id,
        }
    }
}
