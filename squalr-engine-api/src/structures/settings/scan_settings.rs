use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;

#[derive(Deserialize, Serialize)]
pub struct ScanSettings {
    pub results_page_size: u32,
    pub results_read_interval: u32,
    pub table_read_interval: u32,
    pub freeze_interval: u32,
    pub memory_alignment: Option<MemoryAlignment>,
    pub floating_point_tolerance: FloatingPointTolerance,
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
            results_read_interval: 2500,
            table_read_interval: 2500,
            freeze_interval: 50,
            memory_alignment: None,
            floating_point_tolerance: FloatingPointTolerance::default(),
        }
    }
}
