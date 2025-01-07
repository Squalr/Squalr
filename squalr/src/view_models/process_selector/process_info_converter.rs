use crate::ProcessViewData;
use crate::view_models::view_data_converter::ViewDataConverter;
use slint::Image;
use slint::SharedPixelBuffer;
use squalr_engine_processes::process_info::ProcessInfo;

pub struct ProcessInfoConverter;

impl ViewDataConverter<ProcessInfo, ProcessViewData> for ProcessInfoConverter {
    fn convert(
        &self,
        process_info: ProcessInfo,
    ) -> ProcessViewData {
        let icon = process_info.icon.map_or_else(
            || {
                // Create 1x1 transparent image as fallback
                let mut icon_data = SharedPixelBuffer::new(1, 1);
                let icon_data_bytes = icon_data.make_mut_bytes();
                icon_data_bytes.copy_from_slice(&[0, 0, 0, 0]);
                Image::from_rgba8(icon_data)
            },
            |icon| {
                let mut icon_data = SharedPixelBuffer::new(icon.width, icon.height);
                let icon_data_bytes = icon_data.make_mut_bytes();
                icon_data_bytes.copy_from_slice(&icon.bytes_rgba);
                Image::from_rgba8(icon_data)
            },
        );

        ProcessViewData {
            process_id_str: process_info.pid.to_string().into(),
            process_id: process_info.pid.as_u32() as i32,
            name: process_info.name.to_string().into(),
            icon,
        }
    }
}
