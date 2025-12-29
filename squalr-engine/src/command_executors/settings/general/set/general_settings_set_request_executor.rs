use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::general_settings_config::GeneralSettingsConfig;
use squalr_engine_api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use squalr_engine_api::commands::settings::general::set::general_settings_set_response::GeneralSettingsSetResponse;
use std::sync::Arc;

impl EngineCommandRequestExecutor for GeneralSettingsSetRequest {
    type ResponseType = GeneralSettingsSetResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Some(engine_request_delay) = self.engine_request_delay {
            GeneralSettingsConfig::set_engine_request_delay(engine_request_delay);
        }

        GeneralSettingsSetResponse {}
    }
}
