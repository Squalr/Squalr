use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan::element_scan::element_scan_response::ElementScanResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ElementScanRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValue>,
    #[structopt(short = "d", long)]
    pub data_type_ids: Vec<String>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl EngineCommandRequest for ElementScanRequest {
    type ResponseType = ElementScanResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::ElementScan {
            element_scan_request: self.clone(),
        })
    }
}

impl From<ElementScanResponse> for ScanResponse {
    fn from(element_scan_response: ElementScanResponse) -> Self {
        ScanResponse::ElementScan { element_scan_response }
    }
}
