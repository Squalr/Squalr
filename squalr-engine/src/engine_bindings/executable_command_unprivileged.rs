use serde::{Deserialize, Serialize};
use squalr_engine_api::{
    commands::{unprivileged_command::UnprivilegedCommand, unprivileged_command_response::UnprivilegedCommandResponse},
    engine::engine_execution_context::EngineExecutionContext,
};
use std::sync::Arc;

/// Defines data that is sent from the GUI or CLI to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineIngress {
    UnprivilegedCommand(UnprivilegedCommand),
}

pub trait ExecutableCommandUnprivleged {
    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> UnprivilegedCommandResponse;
}
