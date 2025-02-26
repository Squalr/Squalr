use std::sync::Arc;

use crate::commands::settings::list::settings_list_response::SettingsListResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command::EngineCommand, engine_request::EngineRequest};
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_memory::memory_settings::MemorySettings;
use squalr_engine_scanning::scan_settings::ScanSettings;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct SettingsListRequest {
    #[structopt(short = "s", long)]
    scan: bool,
    #[structopt(short = "m", long)]
    memory: bool,
    #[structopt(short = "a", long)]
    list_all: bool,
}

impl EngineRequest for SettingsListRequest {
    type ResponseType = SettingsListResponse;

    fn execute(
        &self,
        _execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType {
        let scan = self.scan | self.list_all;
        let memory = self.memory | self.list_all;

        if scan {
            let scan_config = ScanSettings::get_instance().get_full_config().read().unwrap();
            log::info!("{:?}", scan_config);
        }

        if memory {
            let memory_config = MemorySettings::get_instance().get_full_config().read().unwrap();
            log::info!("{:?}", memory_config);
        }

        SettingsListResponse {}
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::List {
            settings_list_request: self.clone(),
        })
    }
}

impl From<SettingsListResponse> for SettingsResponse {
    fn from(settings_list_response: SettingsListResponse) -> Self {
        SettingsResponse::List { settings_list_response }
    }
}
