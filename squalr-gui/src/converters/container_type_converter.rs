use crate::ContainerTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::structs::container_type::ContainerType;

pub struct ContainerTypeConverter {}

impl ContainerTypeConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ContainerType, ContainerTypeView> for ContainerTypeConverter {
    fn convert_collection(
        &self,
        container_type_list: &Vec<ContainerType>,
    ) -> Vec<ContainerTypeView> {
        container_type_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        container_type: &ContainerType,
    ) -> ContainerTypeView {
        match container_type {
            ContainerType::None => ContainerTypeView::None,
            ContainerType::Array => ContainerTypeView::Array,
            ContainerType::Pointer => ContainerTypeView::Pointer,
        }
    }
}

impl ConvertFromViewData<ContainerType, ContainerTypeView> for ContainerTypeConverter {
    fn convert_from_view_data(
        &self,
        container_type: &ContainerTypeView,
    ) -> ContainerType {
        match container_type {
            ContainerTypeView::None => ContainerType::None,
            ContainerTypeView::Array => ContainerType::Array,
            ContainerTypeView::Pointer => ContainerType::Pointer,
        }
    }
}
