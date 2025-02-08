use crate::commands::settings::settings_command::SettingsCommand;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use uuid::Uuid;

pub fn handle_settings_list(
    cmd: SettingsCommand,
    uuid: Uuid,
) {
    if let SettingsCommand::List { scan, memory, list_all } = cmd {
        let scan = scan | list_all;
        let memory = memory | list_all;

        if scan {
            let scan_config = ScanSettings::get_instance().get_full_config().read().unwrap();
            Logger::get_instance().log(LogLevel::Info, format!("{:?}", scan_config).as_str(), None);
        }

        if memory {
            let memory_config = MemorySettings::get_instance().get_full_config().read().unwrap();
            Logger::get_instance().log(LogLevel::Info, format!("{:?}", memory_config).as_str(), None);
        }
    }
}
