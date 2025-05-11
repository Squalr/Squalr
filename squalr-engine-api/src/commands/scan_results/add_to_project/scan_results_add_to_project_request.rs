use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::add_to_project::scan_results_add_to_project_response::ScanResultsAddToProjectResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::{commands::engine_command::EngineCommand, structures::scan_results::scan_result_base::ScanResultBase};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsAddToProjectRequest {
    #[structopt(short = "s", long)]
    pub scan_results: Vec<ScanResultBase>,
}

impl EngineCommandRequest for ScanResultsAddToProjectRequest {
    type ResponseType = ScanResultsAddToProjectResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::AddToProject {
            results_add_to_project_request: self.clone(),
        })
    }
}

impl From<ScanResultsAddToProjectResponse> for ScanResultsResponse {
    fn from(scan_results_add_to_project_response: ScanResultsAddToProjectResponse) -> Self {
        ScanResultsResponse::AddToProject {
            scan_results_add_to_project_response,
        }
    }
}
