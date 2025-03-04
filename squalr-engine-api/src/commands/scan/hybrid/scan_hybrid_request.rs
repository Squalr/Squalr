use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::hybrid::scan_hybrid_response::ScanHybridResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use serde::{Deserialize, Serialize};
use squalr_engine_common::structures::{data_values::anonymous_value::AnonymousValue, scanning::scan_compare_type::ScanCompareType};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanHybridRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl EngineRequest for ScanHybridRequest {
    type ResponseType = ScanHybridResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::Hybrid {
            scan_hybrid_request: self.clone(),
        })
    }
}

impl From<ScanHybridResponse> for ScanResponse {
    fn from(scan_hybrid_response: ScanHybridResponse) -> Self {
        ScanResponse::Hybrid { scan_hybrid_response }
    }
}
