use crate::{command_executors::engine_command_executor::EngineCommandExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::{engine_command_response::EngineCommandResponse, settings::settings_command::SettingsCommand};
use std::sync::Arc;

impl EngineCommandExecutor for SettingsCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            SettingsCommand::General { general_settings_command } => general_settings_command.execute(engine_privileged_state),
            SettingsCommand::Memory { memory_settings_command } => memory_settings_command.execute(engine_privileged_state),
            SettingsCommand::Scan { scan_settings_command } => scan_settings_command.execute(engine_privileged_state),
        }
    }
}
