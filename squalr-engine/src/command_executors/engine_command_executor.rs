use crate::engine_privileged_state::EnginePrivilegedState;
use interprocess_shell::interprocess_ingress::ExecutableRequest;
use serde::{Serialize, de::DeserializeOwned};
use squalr_engine_api::commands::{engine_command::EngineCommand, engine_response::EngineResponse};
use std::sync::Arc;

pub trait EngineCommandExecutor: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> Self::ResponseType;
}

impl ExecutableRequest<EngineResponse, EnginePrivilegedState> for EngineCommand {
    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> EngineResponse {
        match self {
            EngineCommand::Memory(command) => command.execute(execution_context),
            EngineCommand::Process(command) => command.execute(execution_context),
            EngineCommand::Project(command) => command.execute(execution_context),
            EngineCommand::Results(command) => command.execute(execution_context),
            EngineCommand::Scan(command) => command.execute(execution_context),
            EngineCommand::Settings(command) => command.execute(execution_context),
        }
    }
}
