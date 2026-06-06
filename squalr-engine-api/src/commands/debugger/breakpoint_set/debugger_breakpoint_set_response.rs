use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerBreakpointDescriptor, DebuggerCommandStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerBreakpointSetResponse {
    pub status: DebuggerCommandStatus,
    pub breakpoint: Option<DebuggerBreakpointDescriptor>,
}

impl TypedPrivilegedCommandResponse for DebuggerBreakpointSetResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::BreakpointSet {
            debugger_breakpoint_set_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::BreakpointSet {
            debugger_breakpoint_set_response,
        }) = response
        {
            Ok(debugger_breakpoint_set_response)
        } else {
            Err(response)
        }
    }
}
