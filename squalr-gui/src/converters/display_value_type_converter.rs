use crate::DisplayValueTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::display_value_type::{DisplayContainer, DisplayValueType};

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
            DisplayValueType::Bool(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::Bool,
                DisplayContainer::Array => DisplayValueTypeView::BoolArray,
            },
            DisplayValueType::String(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::Bool,
                DisplayContainer::Array => DisplayValueTypeView::BoolArray,
            },
            DisplayValueType::Binary(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::Binary,
                DisplayContainer::Array => DisplayValueTypeView::BinaryArray,
            },
            DisplayValueType::Decimal(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::Decimal,
                DisplayContainer::Array => DisplayValueTypeView::DecimalArray,
            },
            DisplayValueType::Hexadecimal(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::Hexadecimal,
                DisplayContainer::Array => DisplayValueTypeView::HexadecimalArray,
            },
            DisplayValueType::Address(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::Address,
                DisplayContainer::Array => DisplayValueTypeView::AddressArray,
            },
            DisplayValueType::DataTypeRef(display_container) => match display_container {
                DisplayContainer::None => DisplayValueTypeView::DataTypeRef,
                DisplayContainer::Array => DisplayValueTypeView::DataTypeRefArray,
            },
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
            DisplayValueTypeView::Bool => DisplayValueType::Bool(DisplayContainer::None),
            DisplayValueTypeView::BoolArray => DisplayValueType::Bool(DisplayContainer::Array),
            DisplayValueTypeView::String => DisplayValueType::String(DisplayContainer::None),
            DisplayValueTypeView::StringArray => DisplayValueType::String(DisplayContainer::Array),
            DisplayValueTypeView::Binary => DisplayValueType::Binary(DisplayContainer::None),
            DisplayValueTypeView::BinaryArray => DisplayValueType::Binary(DisplayContainer::Array),
            DisplayValueTypeView::Decimal => DisplayValueType::Decimal(DisplayContainer::None),
            DisplayValueTypeView::DecimalArray => DisplayValueType::Decimal(DisplayContainer::Array),
            DisplayValueTypeView::Hexadecimal => DisplayValueType::Hexadecimal(DisplayContainer::None),
            DisplayValueTypeView::HexadecimalArray => DisplayValueType::Hexadecimal(DisplayContainer::Array),
            DisplayValueTypeView::Address => DisplayValueType::Address(DisplayContainer::None),
            DisplayValueTypeView::AddressArray => DisplayValueType::Address(DisplayContainer::Array),
            DisplayValueTypeView::DataTypeRef => DisplayValueType::DataTypeRef(DisplayContainer::None),
            DisplayValueTypeView::DataTypeRefArray => DisplayValueType::DataTypeRef(DisplayContainer::Array),
            DisplayValueTypeView::Enumeration => DisplayValueType::Enumeration,
        }
    }
}
