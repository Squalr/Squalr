use crate::{command_executors::privileged_command_executor::PrivilegedCommandExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::{privileged_command_response::PrivilegedCommandResponse, settings::settings_command::SettingsCommand};
use std::sync::Arc;

impl PrivilegedCommandExecutor for SettingsCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            SettingsCommand::General { general_settings_command } => general_settings_command.execute(engine_privileged_state),
            SettingsCommand::Memory { memory_settings_command } => memory_settings_command.execute(engine_privileged_state),
            SettingsCommand::Scan { scan_settings_command } => scan_settings_command.execute(engine_privileged_state),
        }
    }
}
