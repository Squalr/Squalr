use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::pointer_scan::summary::pointer_scan_summary_response::PointerScanSummaryResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct PointerScanSummaryRequest {
    #[structopt(short = "i", long)]
    pub session_id: Option<u64>,
}

impl PrivilegedCommandRequest for PointerScanSummaryRequest {
    type ResponseType = PointerScanSummaryResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Summary {
            pointer_scan_summary_request: self.clone(),
        })
    }
}
