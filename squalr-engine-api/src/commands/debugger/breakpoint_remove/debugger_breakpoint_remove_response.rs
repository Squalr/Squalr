use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::DebuggerCommandStatus;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerBreakpointRemoveResponse {
    pub status: DebuggerCommandStatus,
}

impl TypedPrivilegedCommandResponse for DebuggerBreakpointRemoveResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::BreakpointRemove {
            debugger_breakpoint_remove_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::BreakpointRemove {
            debugger_breakpoint_remove_response,
        }) = response
        {
            Ok(debugger_breakpoint_remove_response)
        } else {
            Err(response)
        }
    }
}
