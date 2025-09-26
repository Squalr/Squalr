use crate::ProcessViewData;
use slint::Image;
use slint::SharedPixelBuffer;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;

pub struct OpenedProcessInfoConverter {}

impl OpenedProcessInfoConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<OpenedProcessInfo, ProcessViewData> for OpenedProcessInfoConverter {
    fn convert_collection(
        &self,
        process_info_list: &Vec<OpenedProcessInfo>,
    ) -> Vec<ProcessViewData> {
        process_info_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> ProcessViewData {
        let icon = if let Some(icon_data) = &process_info.get_icon() {
            // Create new buffer and copy the data.
            let mut icon_buffer = SharedPixelBuffer::new(icon_data.get_width(), icon_data.get_height());
            let icon_buffer_bytes = icon_buffer.make_mut_bytes();
            icon_buffer_bytes.copy_from_slice(icon_data.get_bytes_rgba());
            Image::from_rgba8(icon_buffer)
        } else {
            // Create 1x1 transparent image as fallback.
            let mut icon_data = SharedPixelBuffer::new(1, 1);
            let icon_data_bytes = icon_data.make_mut_bytes();
            icon_data_bytes.copy_from_slice(&[0, 0, 0, 0]);
            Image::from_rgba8(icon_data)
        };

        ProcessViewData {
            process_id_str: process_info.get_process_id_raw().to_string().into(),
            process_id: process_info.get_process_id_raw() as i32,
            name: process_info.get_name().to_string().into(),
            icon,
        }
    }
}
