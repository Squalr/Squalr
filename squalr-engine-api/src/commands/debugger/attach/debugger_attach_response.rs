use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerSessionState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerAttachResponse {
    pub status: DebuggerCommandStatus,
    pub session_state: DebuggerSessionState,
    pub active_plugin_id: Option<String>,
}

impl TypedPrivilegedCommandResponse for DebuggerAttachResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::Attach {
            debugger_attach_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::Attach { debugger_attach_response }) = response {
            Ok(debugger_attach_response)
        } else {
            Err(response)
        }
    }
}
