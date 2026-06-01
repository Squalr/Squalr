use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerSessionState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerDetachResponse {
    pub status: DebuggerCommandStatus,
    pub session_state: DebuggerSessionState,
}

impl TypedPrivilegedCommandResponse for DebuggerDetachResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::Detach {
            debugger_detach_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::Detach { debugger_detach_response }) = response {
            Ok(debugger_detach_response)
        } else {
            Err(response)
        }
    }
}
