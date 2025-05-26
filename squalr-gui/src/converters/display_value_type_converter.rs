use crate::DisplayValueTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
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
            DisplayValueType::Binary(_) => DisplayValueTypeView::Binary,
            DisplayValueType::Decimal => DisplayValueTypeView::Decimal,
            DisplayValueType::Hexadecimal(_) => DisplayValueTypeView::Hexadecimal,
            DisplayValueType::Address(_) => DisplayValueTypeView::Address,
            DisplayValueType::ByteArray => DisplayValueTypeView::ByteArray,
            DisplayValueType::DataTypeRef => DisplayValueTypeView::DataTypeRef,
            DisplayValueType::Enumeration => DisplayValueTypeView::Enumeration,
        }
    }
}

impl ConvertFromViewData<DisplayValueType, DisplayValueTypeView> for DisplayValueTypeConverter {
    fn convert_from_view_data(
        &self,
        display_value: &DisplayValueTypeView,
    ) -> DisplayValueType {
        match display_value {
            DisplayValueTypeView::Bool => DisplayValueType::Bool,
            DisplayValueTypeView::String => DisplayValueType::String,
            DisplayValueTypeView::Binary => DisplayValueType::Binary(false),
            DisplayValueTypeView::Decimal => DisplayValueType::Decimal,
            DisplayValueTypeView::Hexadecimal => DisplayValueType::Hexadecimal(false),
            DisplayValueTypeView::Address => DisplayValueType::Address(false),
            DisplayValueTypeView::ByteArray => DisplayValueType::ByteArray,
            DisplayValueTypeView::DataTypeRef => DisplayValueType::DataTypeRef,
            DisplayValueTypeView::Enumeration => DisplayValueType::Enumeration,
        }
    }
}
