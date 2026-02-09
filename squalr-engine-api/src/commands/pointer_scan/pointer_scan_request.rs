use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanRequest {
    #[structopt(short = "a", long)]
    pub target_address: AnonymousValueString,
    #[structopt(short = "t", long)]
    pub pointer_data_type_ref: DataTypeRef,
    #[structopt(short = "d", long)]
    pub max_depth: u64,
    #[structopt(short = "o", long)]
    pub offset_size: u64,
}

impl PrivilegedCommandRequest for PointerScanRequest {
    type ResponseType = PointerScanResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand {
            pointer_scan_request: self.clone(),
        })
    }
}
