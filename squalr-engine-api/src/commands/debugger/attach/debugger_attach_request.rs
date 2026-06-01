use crate::commands::debugger::attach::debugger_attach_response::DebuggerAttachResponse;
use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerAttachRequest {
    pub plugin_id: Option<String>,
}

impl PrivilegedCommandRequest for DebuggerAttachRequest {
    type ResponseType = DebuggerAttachResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::Attach {
            debugger_attach_request: self.clone(),
        })
    }
}

impl From<DebuggerAttachResponse> for DebuggerResponse {
    fn from(debugger_attach_response: DebuggerAttachResponse) -> Self {
        DebuggerResponse::Attach { debugger_attach_response }
    }
}
