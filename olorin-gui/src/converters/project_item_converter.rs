use crate::ProjectViewData;
use slint::{Image, SharedPixelBuffer, ToSharedString};
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use olorin_engine_api::structures::projects::project_info::ProjectInfo;

pub struct ProjectInfoConverter {}

impl ProjectInfoConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<ProjectInfo, ProjectViewData> for ProjectInfoConverter {
    fn convert_collection(
        &self,
        project_info_list: &Vec<ProjectInfo>,
    ) -> Vec<ProjectViewData> {
        project_info_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        project_info: &ProjectInfo,
    ) -> ProjectViewData {
        let icon = if let Some(icon_data) = &project_info.get_project_icon_rgba() {
            // Create new buffer and copy the data
            let mut icon_buffer = SharedPixelBuffer::new(icon_data.get_width(), icon_data.get_height());
            let icon_buffer_bytes = icon_buffer.make_mut_bytes();
            icon_buffer_bytes.copy_from_slice(icon_data.get_bytes_rgba());
            Image::from_rgba8(icon_buffer)
        } else {
            // Create 1x1 transparent image as fallback
            let mut icon_data = SharedPixelBuffer::new(1, 1);
            let icon_data_bytes = icon_data.make_mut_bytes();
            icon_data_bytes.copy_from_slice(&[0, 0, 0, 0]);
            Image::from_rgba8(icon_data)
        };

        ProjectViewData {
            name: project_info.get_name().to_string().into(),
            path: project_info.get_path().to_string_lossy().to_shared_string(),
            icon,
        }
    }
}
