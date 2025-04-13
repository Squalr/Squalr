use crate::{DataTypeIconView, ScanResultViewData};
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;

pub struct ScanResultConverter {}

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
        let address = scan_result.get_address();

        let address_string = if scan_result.is_module() {
            format!("{}+{:X}", scan_result.get_module(), scan_result.get_module_offset())
        } else if address <= u32::MAX as u64 {
            format!("{:08X}", address)
        } else {
            format!("{:016X}", address)
        };

        let data_type_view = match scan_result.get_data_type().get_icon_id().as_ref() {
            "u8" => DataTypeIconView::U8,
            "u16" => DataTypeIconView::U16,
            "u16be" => DataTypeIconView::U16be,
            "u32" => DataTypeIconView::U32,
            "u32be" => DataTypeIconView::U32be,
            "u64" => DataTypeIconView::U64,
            "u64be" => DataTypeIconView::U64be,
            "i8" => DataTypeIconView::I8,
            "i16" => DataTypeIconView::I16,
            "i16be" => DataTypeIconView::I16be,
            "i32" => DataTypeIconView::I32,
            "i32be" => DataTypeIconView::I32be,
            "i64" => DataTypeIconView::I64,
            "i64be" => DataTypeIconView::I64be,
            "f32" => DataTypeIconView::F32,
            "f32be" => DataTypeIconView::F32be,
            "f64" => DataTypeIconView::F64,
            "f64be" => DataTypeIconView::F64be,
            "string" => DataTypeIconView::String,
            "byte_array" => DataTypeIconView::Bytes,
            "bit_field" => DataTypeIconView::Bitfield,
            _ => DataTypeIconView::Unknown,
        };

        let current_value_string = match scan_result.get_recently_read_value() {
            Some(recently_read_value) => recently_read_value.get_value_string(),
            None => match scan_result.get_current_value() {
                Some(current_value) => current_value.get_value_string(),
                None => "??",
            },
        };

        let previous_value_string = match scan_result.get_previous_value() {
            Some(previous_value) => previous_value.get_value_string(),
            None => "??",
        };

        ScanResultViewData {
            address: address_string.into(),
            data_type: data_type_view,
            current_value: current_value_string.into(),
            previous_value: previous_value_string.into(),
            is_frozen: scan_result.get_is_frozen(),
        }
    }

    fn convert_from_view_data(
        &self,
        _scan_result_view_data: &ScanResultViewData,
    ) -> ScanResult {
        panic!("Not implemented.")
    }
}
