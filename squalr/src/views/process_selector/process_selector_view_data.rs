use eframe::egui::{Context, TextureOptions};
use epaint::{ColorImage, TextureHandle};
use squalr_engine_api::structures::processes::{opened_process_info::OpenedProcessInfo, process_icon::ProcessIcon, process_info::ProcessInfo};

pub struct ProcessSelectorViewData {
    pub opened_process: Option<OpenedProcessInfo>,
    pub cached_icon: Option<TextureHandle>,
    pub windowed_processes: Vec<ProcessInfo>,
}

impl ProcessSelectorViewData {
    pub fn new() -> Self {
        Self {
            opened_process: None,
            cached_icon: None,
            windowed_processes: Vec::new(),
        }
    }

    pub fn set_opened_process(
        &mut self,
        context: &Context,
        opened_process: Option<OpenedProcessInfo>,
    ) {
        self.opened_process = opened_process;

        match &self.opened_process {
            Some(opened_proces) => match opened_proces.get_icon() {
                Some(icon) => {
                    self.cached_icon = Some(self.get_or_create_icon(context, opened_proces.get_process_id_raw(), icon));
                }
                None => self.cached_icon = None,
            },
            None => self.cached_icon = None,
        }
    }

    pub fn get_or_create_icon(
        &self,
        context: &Context,
        process_id: u32,
        icon: &ProcessIcon,
    ) -> TextureHandle {
        let size = [icon.get_width() as usize, icon.get_height() as usize];
        let texture_handle = context.load_texture("", ColorImage::from_rgba_unmultiplied(size, icon.get_bytes_rgba()), TextureOptions::default());

        texture_handle
    }
}
