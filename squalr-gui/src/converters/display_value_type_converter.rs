use crate::DisplayValueTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::display_value_type::DisplayValueType;
use squalr_engine_api::structures::structs::container_type::ContainerType;

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
                ContainerType::Pointer => DisplayValueTypeView::BoolPointer,
            },
            DisplayValueType::String(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::String,
                ContainerType::Array => DisplayValueTypeView::StringArray,
                ContainerType::Pointer => DisplayValueTypeView::StringPointer,
            },
            DisplayValueType::Binary(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Binary,
                ContainerType::Array => DisplayValueTypeView::BinaryArray,
                ContainerType::Pointer => DisplayValueTypeView::BinaryPointer,
            },
            DisplayValueType::Decimal(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Decimal,
                ContainerType::Array => DisplayValueTypeView::DecimalArray,
                ContainerType::Pointer => DisplayValueTypeView::DecimalPointer,
            },
            DisplayValueType::Hexadecimal(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Hexadecimal,
                ContainerType::Array => DisplayValueTypeView::HexadecimalArray,
                ContainerType::Pointer => DisplayValueTypeView::HexadecimalPointer,
            },
            DisplayValueType::Address(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Address,
                ContainerType::Array => DisplayValueTypeView::AddressArray,
                ContainerType::Pointer => DisplayValueTypeView::AddressPointer,
            },
            DisplayValueType::DataTypeRef(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::DataTypeRef,
                ContainerType::Array => DisplayValueTypeView::DataTypeRefArray,
                ContainerType::Pointer => DisplayValueTypeView::DataTypeRefPointer,
            },
            DisplayValueType::Enumeration(container_type) => match container_type {
                ContainerType::None => DisplayValueTypeView::Enumeration,
                ContainerType::Array => DisplayValueTypeView::EnumerationArray,
                ContainerType::Pointer => DisplayValueTypeView::EnumerationPointer,
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
            DisplayValueTypeView::BoolPointer => DisplayValueType::Bool(ContainerType::Pointer),
            DisplayValueTypeView::String => DisplayValueType::String(ContainerType::None),
            DisplayValueTypeView::StringArray => DisplayValueType::String(ContainerType::Array),
            DisplayValueTypeView::StringPointer => DisplayValueType::String(ContainerType::Pointer),
            DisplayValueTypeView::Binary => DisplayValueType::Binary(ContainerType::None),
            DisplayValueTypeView::BinaryArray => DisplayValueType::Binary(ContainerType::Array),
            DisplayValueTypeView::BinaryPointer => DisplayValueType::Binary(ContainerType::Pointer),
            DisplayValueTypeView::Decimal => DisplayValueType::Decimal(ContainerType::None),
            DisplayValueTypeView::DecimalArray => DisplayValueType::Decimal(ContainerType::Array),
            DisplayValueTypeView::DecimalPointer => DisplayValueType::Decimal(ContainerType::Pointer),
            DisplayValueTypeView::Hexadecimal => DisplayValueType::Hexadecimal(ContainerType::None),
            DisplayValueTypeView::HexadecimalArray => DisplayValueType::Hexadecimal(ContainerType::Array),
            DisplayValueTypeView::HexadecimalPointer => DisplayValueType::Hexadecimal(ContainerType::Pointer),
            DisplayValueTypeView::Address => DisplayValueType::Address(ContainerType::None),
            DisplayValueTypeView::AddressArray => DisplayValueType::Address(ContainerType::Array),
            DisplayValueTypeView::AddressPointer => DisplayValueType::Address(ContainerType::Pointer),
            DisplayValueTypeView::DataTypeRef => DisplayValueType::DataTypeRef(ContainerType::None),
            DisplayValueTypeView::DataTypeRefArray => DisplayValueType::DataTypeRef(ContainerType::Array),
            DisplayValueTypeView::DataTypeRefPointer => DisplayValueType::DataTypeRef(ContainerType::Pointer),
            DisplayValueTypeView::Enumeration => DisplayValueType::Enumeration(ContainerType::None),
            DisplayValueTypeView::EnumerationArray => DisplayValueType::Enumeration(ContainerType::Array),
            DisplayValueTypeView::EnumerationPointer => DisplayValueType::Enumeration(ContainerType::Pointer),
        }
    }
}
