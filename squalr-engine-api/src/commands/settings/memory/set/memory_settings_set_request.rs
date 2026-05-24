use crate::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemorySettingsSetRequest {
    pub memory_type_none: Option<bool>,
    pub memory_type_private: Option<bool>,
    pub memory_type_image: Option<bool>,
    pub memory_type_mapped: Option<bool>,
    pub required_write: Option<bool>,
    pub required_execute: Option<bool>,
    pub required_copy_on_write: Option<bool>,
    pub excluded_write: Option<bool>,
    pub excluded_execute: Option<bool>,
    pub excluded_copy_on_write: Option<bool>,
    pub start_address: Option<u64>,
    pub end_address: Option<u64>,
    pub only_query_usermode: Option<bool>,
}

impl PrivilegedCommandRequest for MemorySettingsSetRequest {
    type ResponseType = MemorySettingsSetResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
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
