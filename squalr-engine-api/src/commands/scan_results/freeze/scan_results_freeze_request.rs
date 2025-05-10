use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::{commands::engine_command::EngineCommand, structures::scan_results::scan_result_base::ScanResultBase};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsFreezeRequest {
    #[structopt(short = "s", long)]
    pub scan_results: Vec<ScanResultBase>,
    #[structopt(short = "f", long)]
    pub is_frozen: bool,
}

impl EngineCommandRequest for ScanResultsFreezeRequest {
    type ResponseType = ScanResultsFreezeResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::Freeze {
            results_freeze_request: self.clone(),
        })
    }
}

impl From<ScanResultsFreezeResponse> for ScanResultsResponse {
    fn from(scan_results_freeze_response: ScanResultsFreezeResponse) -> Self {
        ScanResultsResponse::Freeze { scan_results_freeze_response }
    }
}
