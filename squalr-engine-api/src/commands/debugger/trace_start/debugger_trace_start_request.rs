use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::trace_start::debugger_trace_start_response::DebuggerTraceStartResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::debugger::{DebuggerDataBreakpointAccess, DebuggerTraceTargetKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerTraceStartRequest {
    pub address: u64,
    pub size_in_bytes: u8,
    pub access: DebuggerDataBreakpointAccess,
    pub label: Option<String>,
    /// Address-directed (data watchpoint on `address`) vs instruction-directed (execute breakpoint at `address`, which
    /// records the memory addresses the instruction accesses). Defaults to address-directed.
    #[serde(default)]
    pub target_kind: DebuggerTraceTargetKind,
}

impl PrivilegedCommandRequest for DebuggerTraceStartRequest {
    type ResponseType = DebuggerTraceStartResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::TraceStart {
            debugger_trace_start_request: self.clone(),
        })
    }
}

impl From<DebuggerTraceStartResponse> for DebuggerResponse {
    fn from(debugger_trace_start_response: DebuggerTraceStartResponse) -> Self {
        DebuggerResponse::TraceStart { debugger_trace_start_response }
    }
}
