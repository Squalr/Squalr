use crate::ContainerTypeView;
use olorin_engine_api::structures::structs::container_type::ContainerType;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;

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
            ContainerType::Array(length) => ContainerTypeView::Array,
            ContainerType::Pointer32 => ContainerTypeView::Pointer32,
            ContainerType::Pointer64 => ContainerTypeView::Pointer64,
        }
    }
}

impl ConvertFromViewData<ContainerType, ContainerTypeView> for ContainerTypeConverter {
    fn convert_from_view_data(
        &self,
        container_type: &ContainerTypeView,
    ) -> ContainerType {
        let JIRA = 69;

        match container_type {
            ContainerTypeView::None => ContainerType::None,
            ContainerTypeView::Array => ContainerType::Array(JIRA),
            ContainerTypeView::Pointer32 => ContainerType::Pointer32,
            ContainerTypeView::Pointer64 => ContainerType::Pointer64,
        }
    }
}
