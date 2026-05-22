use crate::commands::{
    privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse, unprivileged_command::UnprivilegedCommand,
    unprivileged_command_response::UnprivilegedCommandResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandInvocationSource {
    ApiRequest,
    Gui,
    Prompt,
    Tui,
    Cli,
    Internal,
    Unknown(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineCommand {
    Privileged(PrivilegedCommand),
    Unprivileged(UnprivilegedCommand),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineCommandResponse {
    Privileged(PrivilegedCommandResponse),
    Unprivileged(UnprivilegedCommandResponse),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandInvocation {
    invocation_id: u64,
    source: CommandInvocationSource,
    command: EngineCommand,
}

impl CommandInvocation {
    pub fn new(
        invocation_id: u64,
        source: CommandInvocationSource,
        command: EngineCommand,
    ) -> Self {
        Self {
            invocation_id,
            source,
            command,
        }
    }

    pub fn get_invocation_id(&self) -> u64 {
        self.invocation_id
    }

    pub fn get_source(&self) -> &CommandInvocationSource {
        &self.source
    }

    pub fn get_command(&self) -> &EngineCommand {
        &self.command
    }

    pub fn replace_command(
        mut self,
        command: EngineCommand,
    ) -> Self {
        self.command = command;
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandInvocationOutcome {
    invocation: CommandInvocation,
    response: EngineCommandResponse,
}

impl CommandInvocationOutcome {
    pub fn new(
        invocation: CommandInvocation,
        response: EngineCommandResponse,
    ) -> Self {
        Self { invocation, response }
    }

    pub fn get_invocation(&self) -> &CommandInvocation {
        &self.invocation
    }

    pub fn get_response(&self) -> &EngineCommandResponse {
        &self.response
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandInvocationDecision {
    Continue,
    ReplaceCommand { command: EngineCommand },
    Reject { reason: String },
    Respond { response: EngineCommandResponse },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandResponseDecision {
    Continue,
    ReplaceResponse { response: EngineCommandResponse },
    Suppress { reason: String },
}
