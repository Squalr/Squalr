use crate::commands::command_handler::CommandHandler;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsSetRequest {
    #[structopt(name = "setting")]
    setting_command: String,
}

impl CommandHandler for SettingsSetRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        // Parse the setting command
        let (domain_and_setting, new_value) = match self.setting_command.split_once('=') {
            Some(parts) => parts,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Invalid command format. Expected format: domain.setting=value", None);
                return;
            }
        };

        let (domain, setting_name) = match domain_and_setting.split_once('.') {
            Some(parts) => parts,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Invalid setting format. Expected format: domain.setting", None);
                return;
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
                return;
            }
        }
    }
}
