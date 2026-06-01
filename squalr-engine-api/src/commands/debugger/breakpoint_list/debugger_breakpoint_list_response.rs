use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerCommandStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerBreakpointListResponse {
    pub status: DebuggerCommandStatus,
    pub breakpoints: Vec<DebuggerBreakpointDescriptor>,
}

impl TypedPrivilegedCommandResponse for DebuggerBreakpointListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::BreakpointList {
            debugger_breakpoint_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::BreakpointList {
            debugger_breakpoint_list_response,
        }) = response
        {
            Ok(debugger_breakpoint_list_response)
        } else {
            Err(response)
        }
    }
}
