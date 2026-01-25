use squalr_engine_api::structures::{
    data_values::{
        container_type::ContainerType, data_value_interpretation_format::DataValueInterpretationFormat, data_value_interpreter::DataValueInterpreter,
    },
    scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
};

#[derive(Clone)]
pub struct ElementScannerValueViewData {
    pub selected_scan_compare_type: ScanCompareType,
    pub current_scan_value: DataValueInterpreter,
    pub menu_id: String,
}

impl ElementScannerValueViewData {
    pub fn new(menu_id: String) -> Self {
        Self {
            selected_scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            current_scan_value: DataValueInterpreter::new(String::new(), DataValueInterpretationFormat::Decimal, ContainerType::None),
            menu_id,
        }
    }
}
