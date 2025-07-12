use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use olorin_engine_api::commands::settings::memory::list::memory_settings_list_response::MemorySettingsListResponse;
use olorin_engine_memory::config::memory_settings_config::MemorySettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for MemorySettingsListRequest {
    type ResponseType = MemorySettingsListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Ok(memory_settings) = MemorySettingsConfig::get_full_config().read() {
            MemorySettingsListResponse {
                memory_settings: Ok(memory_settings.clone()),
            }
        } else {
            MemorySettingsListResponse {
                memory_settings: Err("Failed to read settings".to_string()),
            }
        }
    }
}
