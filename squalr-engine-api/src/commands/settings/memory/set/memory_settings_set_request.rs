use crate::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{engine_command::EngineCommand, engine_command_request::EngineCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemorySettingsSetRequest {
    #[structopt(name = "setting")]
    pub setting_command: String,
}

impl EngineCommandRequest for MemorySettingsSetRequest {
    type ResponseType = MemorySettingsSetResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::Set {
                memory_settings_set_request: self.clone(),
            },
        })
    }
}

impl From<MemorySettingsSetResponse> for MemorySettingsResponse {
    fn from(memory_settings_set_response: MemorySettingsSetResponse) -> Self {
        MemorySettingsResponse::Set { memory_settings_set_response }
    }
}
