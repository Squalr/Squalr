use crate::plugins::memory_view::PageRetrievalMode;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::{data_types::floating_point_tolerance::FloatingPointTolerance, scanning::memory_read_mode::MemoryReadMode};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct ScanSettings {
    pub page_retrieval_mode: PageRetrievalMode,
    pub results_page_size: u32,
    pub results_read_interval_ms: u64,
    pub project_read_interval_ms: u64,
    #[serde(default = "ScanSettings::default_project_file_system_watch_enabled")]
    pub project_file_system_watch_enabled: bool,
    pub freeze_interval_ms: u64,
    pub memory_alignment: Option<MemoryAlignment>,
    pub memory_read_mode: MemoryReadMode,
    pub floating_point_tolerance: FloatingPointTolerance,
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
            page_retrieval_mode: PageRetrievalMode::FromSettings,
            results_page_size: 22,
            results_read_interval_ms: 200,
            project_read_interval_ms: 200,
            project_file_system_watch_enabled: Self::default_project_file_system_watch_enabled(),
            freeze_interval_ms: 50,
            memory_alignment: Some(MemoryAlignment::Alignment1),
            floating_point_tolerance: FloatingPointTolerance::default(),
            memory_read_mode: MemoryReadMode::ReadBeforeScan,
            is_single_threaded_scan: false,
            debug_perform_validation_scan: false,
        }
    }
}

impl ScanSettings {
    pub fn default_project_file_system_watch_enabled() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::ScanSettings;

    #[test]
    fn deserialize_missing_project_file_system_watch_enabled_defaults_to_enabled() {
        let json = r#"{
            "page_retrieval_mode": "FromSettings",
            "results_page_size": 22,
            "results_read_interval_ms": 200,
            "project_read_interval_ms": 200,
            "freeze_interval_ms": 50,
            "memory_alignment": "Alignment1",
            "memory_read_mode": "ReadBeforeScan",
            "floating_point_tolerance": "epsilon",
            "is_single_threaded_scan": false,
            "debug_perform_validation_scan": false
        }"#;

        let scan_settings: ScanSettings = serde_json::from_str(json).expect("Expected legacy scan settings JSON to deserialize.");

        assert!(scan_settings.project_file_system_watch_enabled);
    }
}
