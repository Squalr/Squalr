use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::register_write::debugger_register_write_response::DebuggerRegisterWriteResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerRegisterWriteRequest {
    pub register_name: String,
    pub value: u64,
}

impl PrivilegedCommandRequest for DebuggerRegisterWriteRequest {
    type ResponseType = DebuggerRegisterWriteResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::RegisterWrite {
            debugger_register_write_request: self.clone(),
        })
    }
}

impl From<DebuggerRegisterWriteResponse> for DebuggerResponse {
    fn from(debugger_register_write_response: DebuggerRegisterWriteResponse) -> Self {
        DebuggerResponse::RegisterWrite {
            debugger_register_write_response,
        }
    }
}
