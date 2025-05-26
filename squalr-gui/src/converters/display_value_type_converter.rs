use crate::DisplayValueTypeView;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::display_value_type::DisplayValueType;

pub struct DisplayValueTypeConverter {}

impl DisplayValueTypeConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DisplayValueType, DisplayValueTypeView> for DisplayValueTypeConverter {
    fn convert_collection(
        &self,
        display_value_list: &Vec<DisplayValueType>,
    ) -> Vec<DisplayValueTypeView> {
        display_value_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        display_value: &DisplayValueType,
    ) -> DisplayValueTypeView {
        match display_value {
            DisplayValueType::Bool => DisplayValueTypeView::Bool,
            DisplayValueType::String => DisplayValueTypeView::String,
            DisplayValueType::Bin => DisplayValueTypeView::Bin,
            DisplayValueType::Dec => DisplayValueTypeView::Dec,
            DisplayValueType::Hex => DisplayValueTypeView::Hex,
            DisplayValueType::Address => DisplayValueTypeView::Address,
            DisplayValueType::ByteArray => DisplayValueTypeView::ByteArray,
            DisplayValueType::DataTypeRef => DisplayValueTypeView::DataTypeRef,
            DisplayValueType::Enumeration => DisplayValueTypeView::Enumeration,
        }
    }
}
