use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_response::ScanSettingsListResponse;
use squalr_engine_api::commands::settings::settings_error::SettingsError;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanSettingsListRequest {
    type ResponseType = ScanSettingsListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Ok(scan_settings) = ScanSettingsConfig::get_full_config().read() {
            ScanSettingsListResponse {
                scan_settings: Ok(scan_settings.clone()),
            }
        } else {
            ScanSettingsListResponse {
                scan_settings: Err(SettingsError::read_failure("scan")),
            }
        }
    }
}
