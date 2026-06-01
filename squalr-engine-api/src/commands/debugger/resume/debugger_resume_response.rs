use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerSessionState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerResumeResponse {
    pub status: DebuggerCommandStatus,
    pub session_state: DebuggerSessionState,
}

impl TypedPrivilegedCommandResponse for DebuggerResumeResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::Resume {
            debugger_resume_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::Resume { debugger_resume_response }) = response {
            Ok(debugger_resume_response)
        } else {
            Err(response)
        }
    }
}
