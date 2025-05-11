use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::{commands::engine_command::EngineCommand, structures::scan_results::scan_result_base::ScanResultBase};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsDeleteRequest {
    #[structopt(short = "s", long)]
    pub scan_results: Vec<ScanResultBase>,
}

impl EngineCommandRequest for ScanResultsDeleteRequest {
    type ResponseType = ScanResultsDeleteResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::Delete {
            results_delete_request: self.clone(),
        })
    }
}

impl From<ScanResultsDeleteResponse> for ScanResultsResponse {
    fn from(scan_results_delete_response: ScanResultsDeleteResponse) -> Self {
        ScanResultsResponse::Delete { scan_results_delete_response }
    }
}
