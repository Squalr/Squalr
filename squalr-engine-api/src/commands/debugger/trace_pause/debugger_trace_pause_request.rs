use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::trace_pause::debugger_trace_pause_response::DebuggerTracePauseResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTracePauseRequest {
    pub trace_session_id: String,
}

impl PrivilegedCommandRequest for DebuggerTracePauseRequest {
    type ResponseType = DebuggerTracePauseResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::TracePause {
            debugger_trace_pause_request: self.clone(),
        })
    }
}

impl From<DebuggerTracePauseResponse> for DebuggerResponse {
    fn from(debugger_trace_pause_response: DebuggerTracePauseResponse) -> Self {
        DebuggerResponse::TracePause { debugger_trace_pause_response }
    }
}
