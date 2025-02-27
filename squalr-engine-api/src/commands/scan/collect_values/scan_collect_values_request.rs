use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanCollectValuesRequest {}

impl EngineRequest for ScanCollectValuesRequest {
    type ResponseType = ScanCollectValuesResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::CollectValues {
            scan_value_collector_request: self.clone(),
        })
    }
}

impl From<ScanCollectValuesResponse> for ScanResponse {
    fn from(scan_value_collector_response: ScanCollectValuesResponse) -> Self {
        ScanResponse::CollectValues { scan_value_collector_response }
    }
}
