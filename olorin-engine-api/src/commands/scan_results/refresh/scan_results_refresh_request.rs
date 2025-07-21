use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::registries::registries::Registries;
use crate::traits::from_string_privileged::FromStringPrivileged;
use crate::{commands::engine_command::EngineCommand, structures::scan_results::scan_result_valued::ScanResultValued};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsRefreshRequest {
    #[structopt(short = "r", long, parse(try_from_str = ScanResultsRefreshRequest::scan_result_valued_from_str))]
    pub scan_results: Vec<ScanResultValued>,
}

impl ScanResultsRefreshRequest {
    fn scan_result_valued_from_str(string: &str) -> Result<ScanResultValued, String> {
        // These registries should be cached on the unprivileged host.
        let JIRA = 420;
        let registries = Registries::new();

        ScanResultValued::from_string_privileged(string, &registries)
    }
}

impl EngineCommandRequest for ScanResultsRefreshRequest {
    type ResponseType = ScanResultsRefreshResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::Refresh {
            results_refresh_request: self.clone(),
        })
    }
}

impl From<ScanResultsRefreshResponse> for ScanResultsResponse {
    fn from(scan_results_refresh_response: ScanResultsRefreshResponse) -> Self {
        ScanResultsResponse::Refresh { scan_results_refresh_response }
    }
}
