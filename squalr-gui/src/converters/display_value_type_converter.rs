use crate::DisplayValueTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
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
            DisplayValueType::Bool(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Bool,
                ContainerType::Array => DisplayValueTypeView::BoolArray,
            },
            DisplayValueType::String(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::String,
                ContainerType::Array => DisplayValueTypeView::StringArray,
            },
            DisplayValueType::Binary(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Binary,
                ContainerType::Array => DisplayValueTypeView::BinaryArray,
            },
            DisplayValueType::Decimal(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Decimal,
                ContainerType::Array => DisplayValueTypeView::DecimalArray,
            },
            DisplayValueType::Hexadecimal(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Hexadecimal,
                ContainerType::Array => DisplayValueTypeView::HexadecimalArray,
            },
            DisplayValueType::Address(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Address,
                ContainerType::Array => DisplayValueTypeView::AddressArray,
            },
            DisplayValueType::DataTypeRef(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::DataTypeRef,
                ContainerType::Array => DisplayValueTypeView::DataTypeRefArray,
            },
            DisplayValueType::Enumeration(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Enumeration,
                ContainerType::Array => DisplayValueTypeView::EnumerationArray,
            },
        }
    }
}

impl ConvertFromViewData<DisplayValueType, DisplayValueTypeView> for DisplayValueTypeConverter {
    fn convert_from_view_data(
        &self,
        display_value: &DisplayValueTypeView,
    ) -> DisplayValueType {
        match display_value {
            DisplayValueTypeView::Bool => DisplayValueType::Bool(ContainerType::None),
            DisplayValueTypeView::BoolArray => DisplayValueType::Bool(ContainerType::Array),
            DisplayValueTypeView::String => DisplayValueType::String(ContainerType::None),
            DisplayValueTypeView::StringArray => DisplayValueType::String(ContainerType::Array),
            DisplayValueTypeView::Binary => DisplayValueType::Binary(ContainerType::None),
            DisplayValueTypeView::BinaryArray => DisplayValueType::Binary(ContainerType::Array),
            DisplayValueTypeView::Decimal => DisplayValueType::Decimal(ContainerType::None),
            DisplayValueTypeView::DecimalArray => DisplayValueType::Decimal(ContainerType::Array),
            DisplayValueTypeView::Hexadecimal => DisplayValueType::Hexadecimal(ContainerType::None),
            DisplayValueTypeView::HexadecimalArray => DisplayValueType::Hexadecimal(ContainerType::Array),
            DisplayValueTypeView::Address => DisplayValueType::Address(ContainerType::None),
            DisplayValueTypeView::AddressArray => DisplayValueType::Address(ContainerType::Array),
            DisplayValueTypeView::DataTypeRef => DisplayValueType::DataTypeRef(ContainerType::None),
            DisplayValueTypeView::DataTypeRefArray => DisplayValueType::DataTypeRef(ContainerType::Array),
            DisplayValueTypeView::Enumeration => DisplayValueType::Enumeration(ContainerType::None),
            DisplayValueTypeView::EnumerationArray => DisplayValueType::Enumeration(ContainerType::Array),
        }
    }
}
