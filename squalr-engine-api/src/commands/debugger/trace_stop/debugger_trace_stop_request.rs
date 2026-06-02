use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::trace_stop::debugger_trace_stop_response::DebuggerTraceStopResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceStopRequest {
    pub trace_session_id: String,
}

impl PrivilegedCommandRequest for DebuggerTraceStopRequest {
    type ResponseType = DebuggerTraceStopResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::TraceStop {
            debugger_trace_stop_request: self.clone(),
        })
    }
}

impl From<DebuggerTraceStopResponse> for DebuggerResponse {
    fn from(debugger_trace_stop_response: DebuggerTraceStopResponse) -> Self {
        DebuggerResponse::TraceStop { debugger_trace_stop_response }
    }
}
