use crate::commands::command_handler::CommandHandler;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::{log_level::LogLevel, logger::Logger};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsListRequest {
    #[structopt(short = "s", long)]
    scan: bool,
    #[structopt(short = "m", long)]
    memory: bool,
    #[structopt(short = "a", long)]
    list_all: bool,
}

impl CommandHandler for SettingsListRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        let scan = self.scan | self.list_all;
        let memory = self.memory | self.list_all;

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
