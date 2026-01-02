use crate::{
    command_executors::{privileged_command_executor::PrivilegedCommandExecutor, privileged_request_executor::PrivilegedCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::{privileged_command_response::PrivilegedCommandResponse, settings::scan::scan_settings_command::ScanSettingsCommand};
use std::sync::Arc;

impl PrivilegedCommandExecutor for ScanSettingsCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            ScanSettingsCommand::List { scan_settings_list_request } => scan_settings_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ScanSettingsCommand::Set { scan_settings_set_request } => scan_settings_set_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
