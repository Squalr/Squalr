use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::execute::scan_execute_response::ScanExecuteResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::{data_values::anonymous_value::AnonymousValue, scanning::scan_memory_read_mode::ScanMemoryReadMode};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanExecuteRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
    #[structopt(short = "m", long)]
    pub memory_read_mode: ScanMemoryReadMode,
}

impl EngineRequest for ScanExecuteRequest {
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
