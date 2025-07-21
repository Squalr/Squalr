use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::commands::scan_results::set_property::scan_results_set_property_response::ScanResultsSetPropertyResponse;
use crate::registries::registries::Registries;
use crate::structures::data_values::data_value::DataValue;
use crate::traits::from_string_privileged::FromStringPrivileged;
use crate::{commands::engine_command::EngineCommand, structures::scan_results::scan_result_base::ScanResultBase};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsSetPropertyRequest {
    #[structopt(short = "s", long)]
    pub scan_results: Vec<ScanResultBase>,
    #[structopt(short = "v", long, parse(try_from_str = ScanResultsSetPropertyRequest::data_value_from_str))]
    pub data_value: DataValue,
    #[structopt(short = "f", long)]
    pub field_namespace: String,
}

impl ScanResultsSetPropertyRequest {
    fn data_value_from_str(string: &str) -> Result<DataValue, String> {
        // These registries should be cached on the unprivileged host.
        let JIRA = 420;
        let registries = Registries::new();

        DataValue::from_string_privileged(string, &registries)
    }
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
