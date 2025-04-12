use crate::MemoryReadModeView;
use slint_mvvm::view_data_converter::ViewDataConverter;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;

pub struct MemoryReadModeConverter;

impl MemoryReadModeConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<MemoryReadMode, MemoryReadModeView> for MemoryReadModeConverter {
    fn convert_collection(
        &self,
        memory_read_mode_list: &Vec<MemoryReadMode>,
    ) -> Vec<MemoryReadModeView> {
        memory_read_mode_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        memory_read_mode: &MemoryReadMode,
    ) -> MemoryReadModeView {
        match memory_read_mode {
            MemoryReadMode::Skip => MemoryReadModeView::Skip,
            MemoryReadMode::ReadBeforeScan => MemoryReadModeView::Prior,
            MemoryReadMode::ReadInterleavedWithScan => MemoryReadModeView::Interleave,
        }
    }

    fn convert_from_view_data(
        &self,
        memory_read_mode_view: &MemoryReadModeView,
    ) -> MemoryReadMode {
        match memory_read_mode_view {
            MemoryReadModeView::Skip => MemoryReadMode::Skip,
            MemoryReadModeView::Prior => MemoryReadMode::ReadBeforeScan,
            MemoryReadModeView::Interleave => MemoryReadMode::ReadInterleavedWithScan,
        }
    }
}
