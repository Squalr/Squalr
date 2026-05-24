use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementScanReport {
    scanned_byte_count: u64,
    processed_region_count: u64,
    result_count: u64,
    committed_deleted_result_count: u64,
    scan_duration: Duration,
}

impl ElementScanReport {
    pub fn new(
        scanned_byte_count: u64,
        processed_region_count: u64,
        result_count: u64,
        committed_deleted_result_count: u64,
        scan_duration: Duration,
    ) -> Self {
        Self {
            scanned_byte_count,
            processed_region_count,
            result_count,
            committed_deleted_result_count,
            scan_duration,
        }
    }

    pub fn get_scanned_byte_count(&self) -> u64 {
        self.scanned_byte_count
    }

    pub fn get_processed_region_count(&self) -> u64 {
        self.processed_region_count
    }

    pub fn get_result_count(&self) -> u64 {
        self.result_count
    }

    pub fn get_committed_deleted_result_count(&self) -> u64 {
        self.committed_deleted_result_count
    }

    pub fn get_scan_duration(&self) -> Duration {
        self.scan_duration
    }
}
