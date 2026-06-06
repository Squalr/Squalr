use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceStopResponse {
    pub status: DebuggerCommandStatus,
    pub trace_session: Option<DebuggerTraceSessionDescriptor>,
    pub instruction_records: Vec<DebuggerTraceInstructionRecord>,
}

impl TypedPrivilegedCommandResponse for DebuggerTraceStopResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::TraceStop {
            debugger_trace_stop_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::TraceStop { debugger_trace_stop_response }) = response {
            Ok(debugger_trace_stop_response)
        } else {
            Err(response)
        }
    }
}
