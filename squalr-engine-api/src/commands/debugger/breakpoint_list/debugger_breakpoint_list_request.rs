use crate::commands::debugger::breakpoint_list::debugger_breakpoint_list_response::DebuggerBreakpointListResponse;
use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerBreakpointListRequest {}

impl PrivilegedCommandRequest for DebuggerBreakpointListRequest {
    type ResponseType = DebuggerBreakpointListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::BreakpointList {
            debugger_breakpoint_list_request: self.clone(),
        })
    }
}

impl From<DebuggerBreakpointListResponse> for DebuggerResponse {
    fn from(debugger_breakpoint_list_response: DebuggerBreakpointListResponse) -> Self {
        DebuggerResponse::BreakpointList {
            debugger_breakpoint_list_response,
        }
    }
}
