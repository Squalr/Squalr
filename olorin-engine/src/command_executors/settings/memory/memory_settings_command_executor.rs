use crate::{
    command_executors::{engine_command_executor::EngineCommandExecutor, engine_request_executor::EngineCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use olorin_engine_api::commands::engine_command_response::TypedEngineCommandResponse;
use olorin_engine_api::commands::{engine_command_response::EngineCommandResponse, settings::memory::memory_settings_command::MemorySettingsCommand};
use std::sync::Arc;

impl EngineCommandExecutor for MemorySettingsCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            MemorySettingsCommand::List { memory_settings_list_request } => memory_settings_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            MemorySettingsCommand::Set { memory_settings_set_request } => memory_settings_set_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
