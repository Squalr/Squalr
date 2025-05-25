use crate::MemoryReadModeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;

pub struct MemoryReadModeConverter {}

impl MemoryReadModeConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<MemoryReadMode, MemoryReadModeView> for MemoryReadModeConverter {
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
            MemoryReadMode::ReadBeforeScan => MemoryReadModeView::ReadBeforeScan,
            MemoryReadMode::ReadInterleavedWithScan => MemoryReadModeView::ReadInterleavedWithScan,
        }
    }
}

impl ConvertFromViewData<MemoryReadMode, MemoryReadModeView> for MemoryReadModeConverter {
    fn convert_from_view_data(
        &self,
        memory_read_mode_view: &MemoryReadModeView,
    ) -> MemoryReadMode {
        match memory_read_mode_view {
            MemoryReadModeView::Skip => MemoryReadMode::Skip,
            MemoryReadModeView::ReadBeforeScan => MemoryReadMode::ReadBeforeScan,
            MemoryReadModeView::ReadInterleavedWithScan => MemoryReadMode::ReadInterleavedWithScan,
        }
    }
}
