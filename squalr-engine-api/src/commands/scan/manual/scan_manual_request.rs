use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::manual::scan_manual_response::ScanManualResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::structures::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::anonymous_value::AnonymousValue;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanManualRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl EngineRequest for ScanManualRequest {
    type ResponseType = ScanManualResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::Manual {
            scan_manual_request: self.clone(),
        })
    }
}

impl From<ScanManualResponse> for ScanResponse {
    fn from(scan_manual_response: ScanManualResponse) -> Self {
        ScanResponse::Manual { scan_manual_response }
    }
}
