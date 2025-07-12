use crate::engine_privileged_state::EnginePrivilegedState;
use serde::{Deserialize, Serialize};
use olorin_engine_api::commands::{engine_command::EngineCommand, engine_command_response::EngineCommandResponse};
use std::sync::Arc;

/// Defines data that is sent from the GUI or CLI to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineIngress {
    EngineCommand(EngineCommand),
}

pub trait ExecutableCommand {
    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> EngineCommandResponse;
}
