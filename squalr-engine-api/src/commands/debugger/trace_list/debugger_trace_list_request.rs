use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::trace_list::debugger_trace_list_response::DebuggerTraceListResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerTraceListRequest {}

impl PrivilegedCommandRequest for DebuggerTraceListRequest {
    type ResponseType = DebuggerTraceListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::TraceList {
            debugger_trace_list_request: self.clone(),
        })
    }
}

impl From<DebuggerTraceListResponse> for DebuggerResponse {
    fn from(debugger_trace_list_response: DebuggerTraceListResponse) -> Self {
        DebuggerResponse::TraceList { debugger_trace_list_response }
    }
}
