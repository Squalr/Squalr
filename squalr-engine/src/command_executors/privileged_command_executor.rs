use crate::{engine_bindings::executable_command_privileged::ExecutableCommandPrivileged, engine_privileged_state::EnginePrivilegedState};
use serde::{Serialize, de::DeserializeOwned};
use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
use std::sync::Arc;

pub trait PrivilegedCommandExecutor: Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> Self::ResponseType;
}

impl ExecutableCommandPrivileged for PrivilegedCommand {
    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> PrivilegedCommandResponse {
        match self {
            PrivilegedCommand::Memory(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Process(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Results(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Scan(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::PointerScan(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::StructScan(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::Settings(command) => command.execute(engine_privileged_state),
            PrivilegedCommand::TrackableTasks(command) => command.execute(engine_privileged_state),
        }
    }
}
