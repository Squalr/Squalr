use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_response::MemorySettingsListResponse;
use squalr_engine_api::commands::settings::settings_error::SettingsError;
use squalr_engine_memory::config::memory_settings_config::MemorySettingsConfig;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for MemorySettingsListRequest {
    type ResponseType = MemorySettingsListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Ok(memory_settings) = MemorySettingsConfig::get_full_config().read() {
            MemorySettingsListResponse {
                memory_settings: Ok(memory_settings.clone()),
            }
        } else {
            MemorySettingsListResponse {
                memory_settings: Err(SettingsError::read_failure("memory")),
            }
        }
    }
}
