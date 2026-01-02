use serde::{Deserialize, Serialize};
use squalr_engine_api::{
    commands::{unprivileged_command::UnprivilegedCommand, unprivileged_command_response::UnprivilegedCommandResponse},
    engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings,
};

/// Defines data that is sent from the GUI or CLI to the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineIngress {
    UnprivilegedCommand(UnprivilegedCommand),
}

pub trait ExecutableCommandUnprivleged {
    fn execute(
        &self,
        engine_api_unprivileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> UnprivilegedCommandResponse;
}
