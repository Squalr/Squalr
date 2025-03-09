use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsListRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,
}

impl EngineCommandRequest for ScanResultsListRequest {
    type ResponseType = ScanResultsListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::List {
            results_list_request: self.clone(),
        })
    }
}

impl From<ScanResultsListResponse> for ScanResultsResponse {
    fn from(scan_results_list_response: ScanResultsListResponse) -> Self {
        ScanResultsResponse::List { scan_results_list_response }
    }
}
