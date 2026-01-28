use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::scan::struct_scan::struct_scan_response::StructScanResponse;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct StructScanRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<AnonymousValueString>,
    #[structopt(short = "d", long)]
    pub data_type_ids: Vec<String>,
    #[structopt(short = "c", long)]
    pub compare_type: ScanCompareType,
}

impl PrivilegedCommandRequest for StructScanRequest {
    type ResponseType = StructScanResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Scan(ScanCommand::StructScan {
            struct_scan_request: self.clone(),
        })
    }
}

impl From<StructScanResponse> for ScanResponse {
    fn from(struct_scan_response: StructScanResponse) -> Self {
        ScanResponse::StructScan { struct_scan_response }
    }
}
