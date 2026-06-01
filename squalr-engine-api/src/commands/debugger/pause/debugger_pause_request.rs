use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::pause::debugger_pause_response::DebuggerPauseResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerPauseRequest {}

impl PrivilegedCommandRequest for DebuggerPauseRequest {
    type ResponseType = DebuggerPauseResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::Pause {
            debugger_pause_request: self.clone(),
        })
    }
}

impl From<DebuggerPauseResponse> for DebuggerResponse {
    fn from(debugger_pause_response: DebuggerPauseResponse) -> Self {
        DebuggerResponse::Pause { debugger_pause_response }
    }
}
