use serde::{Deserialize, Serialize};
use squalr_engine_scanning::results::scan_result::ScanResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsEvent {
    pub scan_results: Vec<ScanResult>,
}
