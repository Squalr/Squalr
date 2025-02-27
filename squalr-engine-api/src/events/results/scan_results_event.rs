use serde::{Deserialize, Serialize};
use squalr_engine_common::structures::scan_result::ScanResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsEvent {
    pub scan_results: Vec<ScanResult>,
}
