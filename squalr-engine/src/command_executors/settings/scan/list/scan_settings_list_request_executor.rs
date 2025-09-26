use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_response::ScanSettingsListResponse;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanSettingsListRequest {
    type ResponseType = ScanSettingsListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Ok(scan_settings) = ScanSettingsConfig::get_full_config().read() {
            ScanSettingsListResponse {
                scan_settings: Ok(scan_settings.clone()),
            }
        } else {
            ScanSettingsListResponse {
                scan_settings: Err("Failed to read settings".to_string()),
            }
        }
    }
}
