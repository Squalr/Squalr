use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan::execute::scan_execute_response::ScanExecuteResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanExecuteRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "d", long)]
    pub data_type_ids: Vec<String>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl EngineCommandRequest for ScanExecuteRequest {
    type ResponseType = ScanExecuteResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::Execute {
            scan_execute_request: self.clone(),
        })
    }
}

impl From<ScanExecuteResponse> for ScanResponse {
    fn from(scan_execute_response: ScanExecuteResponse) -> Self {
        ScanResponse::Execute { scan_execute_response }
    }
}
