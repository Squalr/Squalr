use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use squalr_engine_memory::config::memory_settings_config::MemorySettingsConfig;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for MemorySettingsSetRequest {
    type ResponseType = MemorySettingsSetResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        // Parse the setting command
        let (domain_and_setting, new_value) = match self.setting_command.split_once('=') {
            Some(parts) => parts,
            None => {
                log::error!("Invalid command format. Expected format: domain.setting=value");
                return MemorySettingsSetResponse {};
            }
        };

        let (domain, setting_name) = match domain_and_setting.split_once('.') {
            Some(parts) => parts,
            None => {
                log::error!("Invalid setting format. Expected format: domain.setting");
                return MemorySettingsSetResponse {};
            }
        };

        log::error!("Setting {}.{}={}", domain, setting_name, new_value);

        // Dispatch to the appropriate domain handler
        match domain {
            "memory" => {
                MemorySettingsConfig::update_config_field(setting_name, new_value);
            }
            "scan" => {
                ScanSettingsConfig::update_config_field(setting_name, new_value);
            }
            _ => {
                log::error!("Unknown domain");
                return MemorySettingsSetResponse {};
            }
        }

        MemorySettingsSetResponse {}
    }
}
