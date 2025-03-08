use crate::commands::engine_response::EngineResponse;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result_base::ScanResultBase;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsQueryResponse {
    pub scan_results: Vec<ScanResultBase>,
    pub page_index: u64,
    pub last_page_index: u64,
    pub page_size: u64,
    pub result_count: u64,
    pub total_size_in_bytes: u64,
}

impl TypedEngineResponse for ScanResultsQueryResponse {
    fn to_engine_response(&self) -> EngineResponse {
        EngineResponse::Results(ScanResultsResponse::Query {
            scan_results_query_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineResponse) -> Result<Self, EngineResponse> {
        if let EngineResponse::Results(ScanResultsResponse::Query { scan_results_query_response }) = response {
            Ok(scan_results_query_response)
        } else {
            Err(response)
        }
    }
}
