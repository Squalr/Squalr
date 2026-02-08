use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::general_settings_config::GeneralSettingsConfig;
use squalr_engine_api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use squalr_engine_api::commands::settings::general::list::general_settings_list_response::GeneralSettingsListResponse;
use squalr_engine_api::commands::settings::settings_error::SettingsError;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for GeneralSettingsListRequest {
    type ResponseType = GeneralSettingsListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Ok(general_settings) = GeneralSettingsConfig::get_full_config().read() {
            GeneralSettingsListResponse {
                general_settings: Ok(general_settings.clone()),
            }
        } else {
            GeneralSettingsListResponse {
                general_settings: Err(SettingsError::read_failure("general")),
            }
        }
    }
}
