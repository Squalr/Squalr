use std::sync::Arc;

use crate::commands::engine_request::EngineRequest;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command::EngineCommand, settings::set::settings_set_response::SettingsSetResponse};
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsSetRequest {
    #[structopt(name = "setting")]
    setting_command: String,
}

impl EngineRequest for SettingsSetRequest {
    type ResponseType = SettingsSetResponse;

    fn execute(
        &self,
        _execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType {
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

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Set {
            settings_set_request: self.clone(),
        })
    }
}

impl From<SettingsSetResponse> for SettingsResponse {
    fn from(settings_set_response: SettingsSetResponse) -> Self {
        SettingsResponse::Set { settings_set_response }
    }
}
