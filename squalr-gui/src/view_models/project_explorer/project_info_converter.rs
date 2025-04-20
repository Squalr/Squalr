use crate::ProjectViewData;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;

pub struct ProjectInfoConverter {}

impl ProjectInfoConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<ProjectInfo, ProjectViewData> for ProjectInfoConverter {
    fn convert_collection(
        &self,
        project_info_list: &Vec<ProjectInfo>,
    ) -> Vec<ProjectViewData> {
        return project_info_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        project_info: &ProjectInfo,
    ) -> ProjectViewData {
        ProjectViewData {
            name: project_info.get_name().to_string().into(),
        }
    }

    fn convert_from_view_data(
        &self,
        _: &ProjectViewData,
    ) -> ProjectInfo {
        panic!("Not implemented!");
    }
}
