use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// A request to fetch scan results without reading up-to-date values.
/// For fetching values, either use a ListRequest or pair this with a RefreshRequest.
#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsQueryRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,

    #[serde(default)]
    #[structopt(long = "data-type-filter")]
    pub data_type_filters: Option<Vec<DataTypeRef>>,
}

impl PrivilegedCommandRequest for ScanResultsQueryRequest {
    type ResponseType = ScanResultsQueryResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Results(ScanResultsCommand::Query {
            results_query_request: self.clone(),
        })
    }
}

impl From<ScanResultsQueryResponse> for ScanResultsResponse {
    fn from(scan_results_query_response: ScanResultsQueryResponse) -> Self {
        ScanResultsResponse::Query { scan_results_query_response }
    }
}
