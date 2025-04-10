use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_response::EngineCommandResponse;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{engine_command_request::EngineCommandRequest, settings::memory::list::memory_settings_list_request::MemorySettingsListRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum MemorySettingsCommand {
    List {
        #[structopt(flatten)]
        memory_settings_list_request: MemorySettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        memory_settings_set_request: MemorySettingsSetRequest,
    },
}

impl EngineCommandRequest for MemorySettingsCommand {
    type ResponseType = MemorySettingsResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: self.clone(),
        })
    }
}

impl From<MemorySettingsResponse> for EngineCommandResponse {
    fn from(memory_settings_response: MemorySettingsResponse) -> Self {
        EngineCommandResponse::Settings(SettingsResponse::Memory { memory_settings_response })
    }
}
