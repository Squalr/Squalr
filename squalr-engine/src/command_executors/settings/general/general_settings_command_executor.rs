use crate::{
    command_executors::{engine_command_executor::EngineCommandExecutor, engine_request_executor::EngineCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::engine_command_response::TypedEngineCommandResponse;
use squalr_engine_api::commands::{engine_command_response::EngineCommandResponse, settings::general::general_settings_command::GeneralSettingsCommand};
use std::sync::Arc;

impl EngineCommandExecutor for GeneralSettingsCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            GeneralSettingsCommand::List { general_settings_list_request } => general_settings_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            GeneralSettingsCommand::Set { general_settings_set_request } => general_settings_set_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
