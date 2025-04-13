use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::{data_types::floating_point_tolerance::FloatingPointTolerance, scanning::memory_read_mode::MemoryReadMode};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;

#[derive(Clone, Deserialize, Serialize)]
pub struct ScanSettings {
    pub results_page_size: u32,
    pub results_read_interval: u64,
    pub project_read_interval: u64,
    pub freeze_interval: u64,
    pub memory_alignment: Option<MemoryAlignment>,
    pub floating_point_tolerance: FloatingPointTolerance,
    pub memory_read_mode: MemoryReadMode,
    pub is_single_threaded_scan: bool,
    pub debug_perform_validation_scan: bool,
}

impl fmt::Debug for ScanSettings {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "Settings for scan: {}", json),
            Err(_) => write!(formatter, "Scan config {{ could not serialize to JSON }}"),
        }
    }
}

impl Default for ScanSettings {
    fn default() -> Self {
        Self {
            results_page_size: 22,
            results_read_interval: 200,
            project_read_interval: 200,
            freeze_interval: 50,
            memory_alignment: None,
            floating_point_tolerance: FloatingPointTolerance::default(),
            memory_read_mode: MemoryReadMode::ReadBeforeScan,
            is_single_threaded_scan: false,
            debug_perform_validation_scan: false,
        }
    }
}
