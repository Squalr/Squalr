use crate::scanners::scan_control::ScanControl;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub type ProcessMemoryReadCallback = dyn Fn(&OpenedProcessInfo, u64, &mut [u8]) -> bool + Send + Sync;

#[derive(Clone, Default)]
pub struct ScanExecutionContext {
    scan_control: ScanControl,
    process_memory_reader: Option<Arc<ProcessMemoryReadCallback>>,
}

impl ScanExecutionContext {
    pub fn new(
        cancellation_token: Option<Arc<AtomicBool>>,
        progress_reporter: Option<Arc<dyn Fn(f32) + Send + Sync>>,
        process_memory_reader: Option<Arc<ProcessMemoryReadCallback>>,
    ) -> Self {
        Self {
            scan_control: ScanControl::new(cancellation_token, progress_reporter),
            process_memory_reader,
        }
    }

    pub fn as_scan_control(&self) -> &ScanControl {
        &self.scan_control
    }

    pub fn should_cancel(&self) -> bool {
        self.scan_control.should_cancel()
    }

    pub fn report_progress(
        &self,
        progress: f32,
    ) {
        self.scan_control.report_progress(progress);
    }

    pub fn read_bytes(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        if let Some(process_memory_reader) = &self.process_memory_reader {
            process_memory_reader(process_info, address, values)
        } else {
            false
        }
    }
}
