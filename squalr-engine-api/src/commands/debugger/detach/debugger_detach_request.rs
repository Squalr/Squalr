use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::detach::debugger_detach_response::DebuggerDetachResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerDetachRequest {}

impl PrivilegedCommandRequest for DebuggerDetachRequest {
    type ResponseType = DebuggerDetachResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::Detach {
            debugger_detach_request: self.clone(),
        })
    }
}

impl From<DebuggerDetachResponse> for DebuggerResponse {
    fn from(debugger_detach_response: DebuggerDetachResponse) -> Self {
        DebuggerResponse::Detach { debugger_detach_response }
    }
}
