use crate::{ScanResultViewData, converters::scan_result_ref_converter::ScanResultRefConverter};
use olorin_engine_api::structures::{data_values::display_value_type::DisplayValueType, scan_results::scan_result::ScanResult};
use slint_mvvm::convert_to_view_data::ConvertToViewData;

pub struct ScanResultConverter {}

impl ScanResultConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ScanResult, ScanResultViewData> for ScanResultConverter {
    fn convert_collection(
        &self,
        scan_compare_type_list: &Vec<ScanResult>,
    ) -> Vec<ScanResultViewData> {
        scan_compare_type_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        scan_result: &ScanResult,
    ) -> ScanResultViewData {
        let scan_result_ref = scan_result.get_base_result().get_scan_result_ref();
        let address = scan_result.get_address();

        let address_string = if scan_result.is_module() {
            format!("{}+{:X}", scan_result.get_module(), scan_result.get_module_offset())
        } else if address <= u32::MAX as u64 {
            format!("{:08X}", address)
        } else {
            format!("{:016X}", address)
        };

        let current_value_string = match scan_result.get_recently_read_display_values() {
            Some(recently_read_value) => recently_read_value.get_display_value_string(&DisplayValueType::Decimal),
            None => match scan_result.get_current_display_values() {
                Some(current_value) => current_value.get_display_value_string(&DisplayValueType::Decimal),
                None => "??",
            },
        };

        let previous_value_string = match scan_result.get_previous_display_values() {
            Some(previous_value) => previous_value.get_display_value_string(&DisplayValueType::Decimal),
            None => "??",
        };

        ScanResultViewData {
            scan_result_ref: ScanResultRefConverter {}.convert_to_view_data(scan_result_ref),
            address: address_string.into(),
            current_value: current_value_string.into(),
            previous_value: previous_value_string.into(),
            is_frozen: scan_result.get_is_frozen(),
            icon_id: scan_result.get_icon_id().into(),
        }
    }
}
