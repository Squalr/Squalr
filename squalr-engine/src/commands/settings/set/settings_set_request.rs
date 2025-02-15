use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_request::SettingsRequest;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command::EngineCommand, settings::set::settings_set_response::SettingsSetResponse};
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsSetRequest {
    #[structopt(name = "setting")]
    setting_command: String,
}

impl SettingsRequest for SettingsSetRequest {
    type ResponseType = SettingsSetResponse;

    fn execute(&self) -> Self::ResponseType {
        // Parse the setting command
        let (domain_and_setting, new_value) = match self.setting_command.split_once('=') {
            Some(parts) => parts,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Invalid command format. Expected format: domain.setting=value", None);
                return SettingsSetResponse {};
            }
        };

        let (domain, setting_name) = match domain_and_setting.split_once('.') {
            Some(parts) => parts,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Invalid setting format. Expected format: domain.setting", None);
                return SettingsSetResponse {};
            }
        };

        Logger::get_instance().log(LogLevel::Info, &format!("Setting {}.{}={}", domain, setting_name, new_value), None);

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
                Logger::get_instance().log(LogLevel::Error, "Unknown domain", None);
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
