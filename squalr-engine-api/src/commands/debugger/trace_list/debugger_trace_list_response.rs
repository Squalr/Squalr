use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceListResponse {
    pub status: DebuggerCommandStatus,
    pub trace_sessions: Vec<DebuggerTraceSessionDescriptor>,
    pub instruction_records: Vec<DebuggerTraceInstructionRecord>,
}

impl TypedPrivilegedCommandResponse for DebuggerTraceListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::TraceList {
            debugger_trace_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::TraceList { debugger_trace_list_response }) = response {
            Ok(debugger_trace_list_response)
        } else {
            Err(response)
        }
    }
}
