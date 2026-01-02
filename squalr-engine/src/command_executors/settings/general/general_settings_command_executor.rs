use crate::{
    command_executors::{privileged_command_executor::PrivilegedCommandExecutor, privileged_request_executor::PrivilegedCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::{
    privileged_command_response::PrivilegedCommandResponse, settings::general::general_settings_command::GeneralSettingsCommand,
};
use std::sync::Arc;

impl PrivilegedCommandExecutor for GeneralSettingsCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
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
