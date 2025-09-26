use crate::ScanResultRefViewData;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use slint_mvvm::{convert_from_view_data::ConvertFromViewData, convert_to_view_data::ConvertToViewData};

pub struct ScanResultRefConverter {}

impl ScanResultRefConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ScanResultRef, ScanResultRefViewData> for ScanResultRefConverter {
    fn convert_collection(
        &self,
        scan_result_ref_list: &Vec<ScanResultRef>,
    ) -> Vec<ScanResultRefViewData> {
        scan_result_ref_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        scan_result_ref: &ScanResultRef,
    ) -> ScanResultRefViewData {
        let scan_result_index = scan_result_ref.get_scan_result_index();

        ScanResultRefViewData {
            scan_result_index: scan_result_index as i32,
        }
    }
}

impl ConvertFromViewData<ScanResultRef, ScanResultRefViewData> for ScanResultRefConverter {
    fn convert_from_view_data(
        &self,
        scan_result_ref: &ScanResultRefViewData,
    ) -> ScanResultRef {
        ScanResultRef::new(scan_result_ref.scan_result_index as u64)
    }
}
