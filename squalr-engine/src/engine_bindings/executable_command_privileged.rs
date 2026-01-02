use crate::engine_privileged_state::EnginePrivilegedState;
use serde::{Deserialize, Serialize};
use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
use std::sync::Arc;

/// Defines data that is sent from the GUI or CLI to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineIngress {
    PrivilegedCommand(PrivilegedCommand),
}

pub trait ExecutableCommandPrivileged {
    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> PrivilegedCommandResponse;
}
