use crate::ScanResultDataView;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_scanning::results::scan_result::ScanResult;

pub struct ScanResultConverter;

impl ScanResultConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<ScanResult, ScanResultDataView> for ScanResultConverter {
    fn convert_collection(
        &self,
        scan_compare_type_list: &Vec<ScanResult>,
    ) -> Vec<ScanResultDataView> {
        return scan_compare_type_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        scan_result: &ScanResult,
    ) -> ScanResultDataView {
        ScanResultDataView {
            address: "asdf".into(),
            data_type: crate::DataTypeView::Aob,
            value: "val".into(),
            previous_value: "prev".into(),
        }
    }

    fn convert_from_view_data(
        &self,
        scan_result: &ScanResultDataView,
    ) -> ScanResult {
        panic!("Not implemented.")
    }
}
