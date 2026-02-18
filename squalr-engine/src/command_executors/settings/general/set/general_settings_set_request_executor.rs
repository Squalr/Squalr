use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::general_settings_config::GeneralSettingsConfig;
use squalr_engine_api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use squalr_engine_api::commands::settings::general::set::general_settings_set_response::GeneralSettingsSetResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for GeneralSettingsSetRequest {
    type ResponseType = GeneralSettingsSetResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(debug_engine_request_delay_ms) = self.engine_request_delay {
            GeneralSettingsConfig::set_debug_engine_request_delay_ms(debug_engine_request_delay_ms);
        }

        GeneralSettingsSetResponse {}
    }
}
