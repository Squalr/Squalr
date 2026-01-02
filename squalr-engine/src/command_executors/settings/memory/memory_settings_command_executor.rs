use crate::{
    command_executors::{privileged_command_executor::PrivilegedCommandExecutor, privileged_request_executor::PrivilegedCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::{privileged_command_response::PrivilegedCommandResponse, settings::memory::memory_settings_command::MemorySettingsCommand};
use std::sync::Arc;

impl PrivilegedCommandExecutor for MemorySettingsCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
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
