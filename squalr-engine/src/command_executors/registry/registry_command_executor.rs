use crate::{
    command_executors::{privileged_command_executor::PrivilegedCommandExecutor, privileged_request_executor::PrivilegedCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::{
    privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
    registry::registry_command::RegistryCommand,
};
use std::sync::Arc;

impl PrivilegedCommandExecutor for RegistryCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> Self::ResponseType {
        match self {
            RegistryCommand::GetSnapshot { registry_get_snapshot_request } => registry_get_snapshot_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            RegistryCommand::SetProjectSymbols {
                registry_set_project_symbols_request,
            } => registry_set_project_symbols_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
