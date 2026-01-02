use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::scan_results::delete::scan_results_delete_response::ScanResultsDeleteResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::{commands::privileged_command::PrivilegedCommand, structures::scan_results::scan_result_ref::ScanResultRef};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsDeleteRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<ScanResultRef>,
}

impl PrivilegedCommandRequest for ScanResultsDeleteRequest {
    type ResponseType = ScanResultsDeleteResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Results(ScanResultsCommand::Delete {
            results_delete_request: self.clone(),
        })
    }
}

impl From<ScanResultsDeleteResponse> for ScanResultsResponse {
    fn from(scan_results_delete_response: ScanResultsDeleteResponse) -> Self {
        ScanResultsResponse::Delete { scan_results_delete_response }
    }
}
