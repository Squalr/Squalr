use crate::commands::debugger::breakpoint_set::debugger_breakpoint_set_response::DebuggerBreakpointSetResponse;
use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::structures::debugger::DebuggerBreakpointKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerBreakpointSetRequest {
    pub address: u64,
    pub kind: DebuggerBreakpointKind,
    pub label: Option<String>,
}

impl PrivilegedCommandRequest for DebuggerBreakpointSetRequest {
    type ResponseType = DebuggerBreakpointSetResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::BreakpointSet {
            debugger_breakpoint_set_request: self.clone(),
        })
    }
}

impl From<DebuggerBreakpointSetResponse> for DebuggerResponse {
    fn from(debugger_breakpoint_set_response: DebuggerBreakpointSetResponse) -> Self {
        DebuggerResponse::BreakpointSet {
            debugger_breakpoint_set_response,
        }
    }
}
