use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan::new::scan_new_response::ScanNewResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::{engine_command::EngineCommand, scan::scan_command::ScanCommand};
use crate::structures::scanning::parameters::user_scan_parameters_local::UserScanParametersLocal;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanNewRequest {
    #[structopt(short = "d", long, use_delimiter = true)]
    pub user_scan_parameters_local: Vec<UserScanParametersLocal>,
}

impl EngineCommandRequest for ScanNewRequest {
    type ResponseType = ScanNewResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::New {
            scan_new_request: self.clone(),
        })
    }
}

impl From<ScanNewResponse> for ScanResponse {
    fn from(scan_new_response: ScanNewResponse) -> Self {
        ScanResponse::New { scan_new_response }
    }
}
