use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::struct_scan::struct_scan_command::StructScanCommand;
use crate::commands::struct_scan::struct_scan_response::StructScanResponse;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructScanRequest {
    pub scan_value: Option<AnonymousValueString>,
    pub data_type_ids: Vec<String>,
    pub compare_type: ScanCompareType,
}

impl PrivilegedCommandRequest for StructScanRequest {
    type ResponseType = StructScanResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::StructScan(StructScanCommand {
            struct_scan_request: self.clone(),
        })
    }
}
