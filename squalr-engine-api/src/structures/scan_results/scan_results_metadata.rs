use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsMetadata {
    pub result_count: u64,
    pub total_size_in_bytes: u64,
}
