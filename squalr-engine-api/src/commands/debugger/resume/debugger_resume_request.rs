use crate::commands::debugger::debugger_command::DebuggerCommand;
use crate::commands::debugger::debugger_response::DebuggerResponse;
use crate::commands::debugger::resume::debugger_resume_response::DebuggerResumeResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DebuggerResumeRequest {}

impl PrivilegedCommandRequest for DebuggerResumeRequest {
    type ResponseType = DebuggerResumeResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Debugger(DebuggerCommand::Resume {
            debugger_resume_request: self.clone(),
        })
    }
}

impl From<DebuggerResumeResponse> for DebuggerResponse {
    fn from(debugger_resume_response: DebuggerResumeResponse) -> Self {
        DebuggerResponse::Resume { debugger_resume_response }
    }
}
