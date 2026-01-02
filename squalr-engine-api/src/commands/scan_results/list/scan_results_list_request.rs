use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
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

impl PrivilegedCommandRequest for ScanResultsListRequest {
    type ResponseType = ScanResultsListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Results(ScanResultsCommand::List {
            results_list_request: self.clone(),
        })
    }
}

impl From<ScanResultsListResponse> for ScanResultsResponse {
    fn from(scan_results_list_response: ScanResultsListResponse) -> Self {
        ScanResultsResponse::List { scan_results_list_response }
    }
}
