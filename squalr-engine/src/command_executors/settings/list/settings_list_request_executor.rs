use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::list::settings_list_request::SettingsListRequest;
use squalr_engine_api::commands::settings::list::settings_list_response::SettingsListResponse;
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use std::sync::Arc;

impl EngineCommandRequestExecutor for SettingsListRequest {
    type ResponseType = SettingsListResponse;

    fn execute(
        &self,
        _execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let scan = self.scan | self.list_all;
        let memory = self.memory | self.list_all;

        if scan {
            let scan_config = ScanSettings::get_instance().get_full_config().read().unwrap();
            log::info!("{:?}", scan_config);
        }

        if memory {
            let memory_config = MemorySettings::get_instance().get_full_config().read().unwrap();
            log::info!("{:?}", memory_config);
        }

        SettingsListResponse {}
    }
}
