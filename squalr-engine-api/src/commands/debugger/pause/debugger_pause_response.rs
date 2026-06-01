use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerSessionState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerPauseResponse {
    pub status: DebuggerCommandStatus,
    pub session_state: DebuggerSessionState,
}

impl TypedPrivilegedCommandResponse for DebuggerPauseResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::Pause {
            debugger_pause_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::Pause { debugger_pause_response }) = response {
            Ok(debugger_pause_response)
        } else {
            Err(response)
        }
    }
}
