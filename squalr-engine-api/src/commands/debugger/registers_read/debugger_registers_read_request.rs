use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::registers_read::debugger_registers_read_response::DebuggerRegistersReadResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerRegistersReadRequest {}

impl PrivilegedCommandRequest for DebuggerRegistersReadRequest {
    type ResponseType = DebuggerRegistersReadResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::RegistersRead {
            debugger_registers_read_request: self.clone(),
        })
    }
}

impl From<DebuggerRegistersReadResponse> for DebuggerResponse {
    fn from(debugger_registers_read_response: DebuggerRegistersReadResponse) -> Self {
        DebuggerResponse::RegistersRead {
            debugger_registers_read_response,
        }
    }
}
