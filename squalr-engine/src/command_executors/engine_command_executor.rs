use crate::{engine_bindings::engine_ingress::ExecutableCommand, engine_privileged_state::EnginePrivilegedState};
use serde::{Serialize, de::DeserializeOwned};
use squalr_engine_api::commands::{engine_command::EngineCommand, engine_command_response::EngineCommandResponse};
use std::sync::Arc;

pub trait EngineCommandExecutor: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> Self::ResponseType;
}

impl ExecutableCommand for EngineCommand {
    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> EngineCommandResponse {
        match self {
            EngineCommand::Memory(command) => command.execute(engine_privileged_state),
            EngineCommand::Process(command) => command.execute(engine_privileged_state),
            EngineCommand::Project(command) => command.execute(engine_privileged_state),
            EngineCommand::ProjectItems(command) => command.execute(engine_privileged_state),
            EngineCommand::Results(command) => command.execute(engine_privileged_state),
            EngineCommand::Scan(command) => command.execute(engine_privileged_state),
            EngineCommand::Settings(command) => command.execute(engine_privileged_state),
            EngineCommand::TrackableTasks(command) => command.execute(engine_privileged_state),
        }
    }
}
