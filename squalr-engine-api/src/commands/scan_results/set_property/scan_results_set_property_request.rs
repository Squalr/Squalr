use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use crate::{commands::engine_command::EngineCommand, structures::data_values::anonymous_value::AnonymousValue};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsSetPropertyRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<ScanResultRef>,
    #[structopt(short = "v", long)]
    pub anonymous_value: AnonymousValue,
    #[structopt(short = "f", long)]
    pub field_namespace: String,
}

impl EngineCommandRequest for ScanResultsSetPropertyRequest {
    type ResponseType = ScanResultsSetPropertyResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::SetProperty {
            results_set_property_request: self.clone(),
        })
    }
}

impl From<ScanResultsSetPropertyResponse> for ScanResultsResponse {
    fn from(scan_results_set_property_response: ScanResultsSetPropertyResponse) -> Self {
        ScanResultsResponse::SetProperty {
            scan_results_set_property_response,
        }
    }
}
