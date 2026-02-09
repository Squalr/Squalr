use crate::engine_bindings::executable_command_unprivileged::ExecutableCommandUnprivleged;
use serde::{Serialize, de::DeserializeOwned};
use squalr_engine_api::{
    commands::{unprivileged_command::UnprivilegedCommand, unprivileged_command_response::UnprivilegedCommandResponse},
    engine::engine_execution_context::EngineExecutionContext,
};
use std::sync::Arc;

pub trait UnprivilegedCommandExecutor: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> Self::ResponseType;
}

impl ExecutableCommandUnprivleged for UnprivilegedCommand {
    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> UnprivilegedCommandResponse {
        match self {
            UnprivilegedCommand::Project(command) => command.execute(engine_unprivileged_state),
            UnprivilegedCommand::ProjectItems(command) => command.execute(engine_unprivileged_state),
        }
    }
}
