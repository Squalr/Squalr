use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::engine_command_response::TypedEngineCommandResponse;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result::ScanResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanResultsQueryResponse {
    pub scan_results: Vec<ScanResult>,
    pub page_index: u64,
    pub last_page_index: u64,
    pub page_size: u64,
    pub result_count: u64,
    pub total_size_in_bytes: u64,
}

impl TypedEngineCommandResponse for ScanResultsQueryResponse {
    fn to_engine_response(&self) -> EngineCommandResponse {
        EngineCommandResponse::Results(ScanResultsResponse::Query {
            scan_results_query_response: self.clone(),
        })
    }

    fn from_engine_response(response: EngineCommandResponse) -> Result<Self, EngineCommandResponse> {
        if let EngineCommandResponse::Results(ScanResultsResponse::Query { scan_results_query_response }) = response {
            Ok(scan_results_query_response)
        } else {
            Err(response)
        }
    }
}
