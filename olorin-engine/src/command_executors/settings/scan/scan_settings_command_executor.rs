use crate::{
    command_executors::{engine_command_executor::EngineCommandExecutor, engine_request_executor::EngineCommandRequestExecutor},
    engine_privileged_state::EnginePrivilegedState,
};
use olorin_engine_api::commands::engine_command_response::TypedEngineCommandResponse;
use olorin_engine_api::commands::{engine_command_response::EngineCommandResponse, settings::scan::scan_settings_command::ScanSettingsCommand};
use std::sync::Arc;

impl EngineCommandExecutor for ScanSettingsCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
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
