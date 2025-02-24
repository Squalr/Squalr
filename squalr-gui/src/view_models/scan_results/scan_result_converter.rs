use crate::ScanResultViewData;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_scanning::results::scan_result::ScanResult;

pub struct ScanResultConverter;

impl ScanResultConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<ScanResult, ScanResultViewData> for ScanResultConverter {
    fn convert_collection(
        &self,
        scan_compare_type_list: &Vec<ScanResult>,
    ) -> Vec<ScanResultViewData> {
        return scan_compare_type_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        scan_result: &ScanResult,
    ) -> ScanResultViewData {
        let base_address = scan_result.get_base_address();
        let module = scan_result.get_module();

        let address_string = if module.is_empty() {
            format!("0x{:X}", base_address)
        } else {
            format!("{}+0x{:X}", module, base_address)
        };

        ScanResultViewData {
            address: address_string.into(),
            data_type: crate::DataTypeView::Aob,
            current_value: scan_result.get_current_value().to_value_string().into(),
            previous_value: scan_result.get_previous_value().to_value_string().into(),
        }
    }

    fn convert_from_view_data(
        &self,
        _scan_result_view_data: &ScanResultViewData,
    ) -> ScanResult {
        panic!("Not implemented.")
    }
}
