use crate::{
    command_executors::{engine_command_executor::EngineCommandExecutor, engine_request_executor::EngineRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::{
    engine_response::{EngineResponse, TypedEngineResponse},
    settings::settings_command::SettingsCommand,
};
use std::sync::Arc;

impl EngineCommandExecutor for SettingsCommand {
    type ResponseType = EngineResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            SettingsCommand::List { settings_list_request } => settings_list_request
                .execute(execution_context)
                .to_engine_response(),
            SettingsCommand::Set { settings_set_request } => settings_set_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
