use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsFreezeRequest {
    pub scan_result_refs: Vec<ScanResultRef>,
    pub is_frozen: bool,
}

impl PrivilegedCommandRequest for ScanResultsFreezeRequest {
    type ResponseType = ScanResultsFreezeResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Results(ScanResultsCommand::Freeze {
            results_freeze_request: self.clone(),
        })
    }
}

impl From<ScanResultsFreezeResponse> for ScanResultsResponse {
    fn from(scan_results_freeze_response: ScanResultsFreezeResponse) -> Self {
        ScanResultsResponse::Freeze { scan_results_freeze_response }
    }
}
