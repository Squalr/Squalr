use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceStartResponse {
    pub status: DebuggerCommandStatus,
    pub trace_session: Option<DebuggerTraceSessionDescriptor>,
    pub instruction_records: Vec<DebuggerTraceInstructionRecord>,
}

impl TypedPrivilegedCommandResponse for DebuggerTraceStartResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::TraceStart {
            debugger_trace_start_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::TraceStart { debugger_trace_start_response }) = response {
            Ok(debugger_trace_start_response)
        } else {
            Err(response)
        }
    }
}
