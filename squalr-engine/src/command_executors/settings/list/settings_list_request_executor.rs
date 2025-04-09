use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::list::settings_list_request::SettingsListRequest;
use squalr_engine_api::commands::settings::list::settings_list_response::SettingsListResponse;
use squalr_engine_memory::config::memory_settings_config::MemorySettingsConfig;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for SettingsListRequest {
    type ResponseType = SettingsListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let scan = self.scan | self.list_all;
        let memory = self.memory | self.list_all;

        if scan {
            if let Ok(scan_config) = ScanSettingsConfig::get_full_config().read() {
                log::info!("{:?}", scan_config);
            }
        }

        if memory {
            if let Ok(memory_config) = MemorySettingsConfig::get_full_config().read() {
                log::info!("{:?}", memory_config);
            }
        }

        SettingsListResponse {}
    }
}
