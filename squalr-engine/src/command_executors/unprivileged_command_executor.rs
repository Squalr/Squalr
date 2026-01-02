use crate::engine_bindings::executable_command_unprivileged::ExecutableCommandUnprivleged;
use serde::{Serialize, de::DeserializeOwned};
use squalr_engine_api::{
    commands::{unprivileged_command::UnprivilegedCommand, unprivileged_command_response::UnprivilegedCommandResponse},
    engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings,
};

pub trait UnprivilegedCommandExecutor: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_api_unprivileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> Self::ResponseType;
}

impl ExecutableCommandUnprivleged for UnprivilegedCommand {
    fn execute(
        &self,
        engine_api_unprivileged_bindings: &dyn EngineApiUnprivilegedBindings,
    ) -> UnprivilegedCommandResponse {
        match self {
            UnprivilegedCommand::Project(command) => command.execute(engine_api_unprivileged_bindings),
            UnprivilegedCommand::ProjectItems(command) => command.execute(engine_api_unprivileged_bindings),
        }
    }
}
