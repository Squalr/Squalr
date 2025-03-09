use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::set::settings_set_request::SettingsSetRequest;
use squalr_engine_api::commands::settings::set::settings_set_response::SettingsSetResponse;
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use std::sync::Arc;

impl EngineCommandRequestExecutor for SettingsSetRequest {
    type ResponseType = SettingsSetResponse;

    fn execute(
        &self,
        _execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        // Parse the setting command
        let (domain_and_setting, new_value) = match self.setting_command.split_once('=') {
            Some(parts) => parts,
            None => {
                log::error!("Invalid command format. Expected format: domain.setting=value");
                return SettingsSetResponse {};
            }
        };

        let (domain, setting_name) = match domain_and_setting.split_once('.') {
            Some(parts) => parts,
            None => {
                log::error!("Invalid setting format. Expected format: domain.setting");
                return SettingsSetResponse {};
            }
        };

        log::error!("Setting {}.{}={}", domain, setting_name, new_value);

        // Dispatch to the appropriate domain handler
        match domain {
            "memory" => {
                let memory_settings = MemorySettings::get_instance();
                memory_settings.update_config_field(setting_name, new_value);
            }
            "scan" => {
                let scan_settings = ScanSettings::get_instance();
                scan_settings.update_config_field(setting_name, new_value);
            }
            _ => {
                log::error!("Unknown domain");
                return SettingsSetResponse {};
            }
        }

        SettingsSetResponse {}
    }
}
