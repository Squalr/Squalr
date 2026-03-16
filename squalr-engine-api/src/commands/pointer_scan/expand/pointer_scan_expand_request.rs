use crate::commands::pointer_scan::expand::pointer_scan_expand_response::PointerScanExpandResponse;
use crate::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanExpandRequest {
    #[structopt(short = "i", long)]
    pub session_id: u64,
    #[structopt(long)]
    pub parent_node_id: Option<u64>,
}

impl PrivilegedCommandRequest for PointerScanExpandRequest {
    type ResponseType = PointerScanExpandResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::PointerScan(PointerScanCommand::Expand {
            pointer_scan_expand_request: self.clone(),
        })
    }
}
