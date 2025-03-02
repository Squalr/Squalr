use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::structures::scan_results::scan_result::ScanResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsListResponse {
    pub scan_results: Vec<ScanResult>,
    pub page_index: u64,
    pub last_page_index: u64,
    pub page_size: u64,
    pub result_count: u64,
    pub total_size_in_bytes: u64,
}

impl TypedEngineResponse for ScanResultsListResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Results(ScanResultsResponse::List {
            scan_results_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Results(ScanResultsResponse::List { scan_results_list_response }) = response {
            Ok(scan_results_list_response)
        } else {
            Err(response)
        }
    }
}
