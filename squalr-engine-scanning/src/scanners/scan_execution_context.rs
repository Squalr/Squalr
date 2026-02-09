use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub type ProcessMemoryReadCallback = dyn Fn(&OpenedProcessInfo, u64, &mut [u8]) -> bool + Send + Sync;

#[derive(Clone, Default)]
pub struct ScanExecutionContext {
    cancellation_token: Option<Arc<AtomicBool>>,
    progress_reporter: Option<Arc<dyn Fn(f32) + Send + Sync>>,
    process_memory_reader: Option<Arc<ProcessMemoryReadCallback>>,
}

impl ScanExecutionContext {
    pub fn new(
        cancellation_token: Option<Arc<AtomicBool>>,
        progress_reporter: Option<Arc<dyn Fn(f32) + Send + Sync>>,
        process_memory_reader: Option<Arc<ProcessMemoryReadCallback>>,
    ) -> Self {
        Self {
            cancellation_token,
            progress_reporter,
            process_memory_reader,
        }
    }

    pub fn should_cancel(&self) -> bool {
        self.cancellation_token
            .as_ref()
            .map(|cancellation_token| cancellation_token.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    pub fn report_progress(
        &self,
        progress: f32,
    ) {
        if let Some(progress_reporter) = &self.progress_reporter {
            progress_reporter(progress);
        }
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
