use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{privileged_command_request::PrivilegedCommandRequest, settings::memory::list::memory_settings_list_request::MemorySettingsListRequest};
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

impl PrivilegedCommandRequest for MemorySettingsCommand {
    type ResponseType = MemorySettingsResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: self.clone(),
        })
    }
}

impl From<MemorySettingsResponse> for PrivilegedCommandResponse {
    fn from(memory_settings_response: MemorySettingsResponse) -> Self {
        PrivilegedCommandResponse::Settings(SettingsResponse::Memory { memory_settings_response })
    }
}
