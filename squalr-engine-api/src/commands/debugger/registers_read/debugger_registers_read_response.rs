use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::structures::debugger::{DebuggerCommandStatus, DebuggerRegisterSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebuggerRegistersReadResponse {
    pub status: DebuggerCommandStatus,
    pub register_snapshot: Option<DebuggerRegisterSnapshot>,
}

impl TypedPrivilegedCommandResponse for DebuggerRegistersReadResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Debugger(DebuggerResponse::RegistersRead {
            debugger_registers_read_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Debugger(DebuggerResponse::RegistersRead {
            debugger_registers_read_response,
        }) = response
        {
            Ok(debugger_registers_read_response)
        } else {
            Err(response)
        }
    }
}
