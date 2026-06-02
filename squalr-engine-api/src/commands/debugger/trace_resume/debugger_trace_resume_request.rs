use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::trace_resume::debugger_trace_resume_response::DebuggerTraceResumeResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceResumeRequest {
    pub trace_session_id: String,
}

impl PrivilegedCommandRequest for DebuggerTraceResumeRequest {
    type ResponseType = DebuggerTraceResumeResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::TraceResume {
            debugger_trace_resume_request: self.clone(),
        })
    }
}

impl From<DebuggerTraceResumeResponse> for DebuggerResponse {
    fn from(debugger_trace_resume_response: DebuggerTraceResumeResponse) -> Self {
        DebuggerResponse::TraceResume {
            debugger_trace_resume_response,
        }
    }
}
