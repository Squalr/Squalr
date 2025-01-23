use crate::ProcessViewData;
use slint::Image;
use slint::SharedPixelBuffer;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_processes::process_info::ProcessInfo;

pub struct ProcessInfoConverter;

impl ProcessInfoConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<ProcessInfo, ProcessViewData> for ProcessInfoConverter {
    fn convert_collection(
        &self,
        process_info_list: &Vec<ProcessInfo>,
    ) -> Vec<ProcessViewData> {
        return process_info_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        process_info: &ProcessInfo,
    ) -> ProcessViewData {
        let icon = if let Some(icon_data) = &process_info.icon {
            // Create new buffer and copy the data
            let mut icon_buffer = SharedPixelBuffer::new(icon_data.width, icon_data.height);
            let icon_buffer_bytes = icon_buffer.make_mut_bytes();
            icon_buffer_bytes.copy_from_slice(&icon_data.bytes_rgba);
            Image::from_rgba8(icon_buffer)
        } else {
            // Create 1x1 transparent image as fallback
            let mut icon_data = SharedPixelBuffer::new(1, 1);
            let icon_data_bytes = icon_data.make_mut_bytes();
            icon_data_bytes.copy_from_slice(&[0, 0, 0, 0]);
            Image::from_rgba8(icon_data)
        };

        ProcessViewData {
            process_id_str: process_info.pid.to_string().into(),
            process_id: process_info.pid as i32,
            name: process_info.name.to_string().into(),
            icon,
        }
    }

    fn convert_from_view_data(
        &self,
        _: &ProcessViewData,
    ) -> ProcessInfo {
        panic!("Not implemented!");
    }
}
