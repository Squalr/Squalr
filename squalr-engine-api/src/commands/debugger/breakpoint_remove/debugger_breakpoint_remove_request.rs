use crate::commands::debugger::breakpoint_remove::debugger_breakpoint_remove_response::DebuggerBreakpointRemoveResponse;
use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerBreakpointRemoveRequest {
    pub breakpoint_id: String,
}

impl PrivilegedCommandRequest for DebuggerBreakpointRemoveRequest {
    type ResponseType = DebuggerBreakpointRemoveResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::BreakpointRemove {
            debugger_breakpoint_remove_request: self.clone(),
        })
    }
}

impl From<DebuggerBreakpointRemoveResponse> for DebuggerResponse {
    fn from(debugger_breakpoint_remove_response: DebuggerBreakpointRemoveResponse) -> Self {
        DebuggerResponse::BreakpointRemove {
            debugger_breakpoint_remove_response,
        }
    }
}
