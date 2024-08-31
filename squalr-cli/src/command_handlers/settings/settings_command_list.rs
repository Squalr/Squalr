use crate::command_handlers::settings::settings_command::SettingsCommand;
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;

pub fn handle_settings_list(cmd: &mut SettingsCommand) {
    if let SettingsCommand::List {} = cmd {
        let memory_config = MemorySettings::get_instance().get_full_config().read().unwrap();
        let scan_config = ScanSettings::get_instance().get_full_config().read().unwrap();

        Logger::get_instance().log(LogLevel::Info, format!("{:?}", memory_config).as_str(), None);
        Logger::get_instance().log(LogLevel::Info, format!("{:?}", scan_config).as_str(), None);
    }
}
