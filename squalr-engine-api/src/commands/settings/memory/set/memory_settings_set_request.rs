use crate::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use crate::commands::settings::settings_command::SettingsCommand;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Default, Serialize, Deserialize)]
pub struct MemorySettingsSetRequest {
    #[structopt(long)]
    pub memory_type_none: Option<bool>,
    #[structopt(long)]
    pub memory_type_private: Option<bool>,
    #[structopt(long)]
    pub memory_type_image: Option<bool>,
    #[structopt(long)]
    pub memory_type_mapped: Option<bool>,
    #[structopt(long)]
    pub required_write: Option<bool>,
    #[structopt(long)]
    pub required_execute: Option<bool>,
    #[structopt(long)]
    pub required_copy_on_write: Option<bool>,
    #[structopt(long)]
    pub excluded_write: Option<bool>,
    #[structopt(long)]
    pub excluded_execute: Option<bool>,
    #[structopt(long)]
    pub excluded_copy_on_write: Option<bool>,
    #[structopt(long)]
    pub start_address: Option<u64>,
    #[structopt(long)]
    pub end_address: Option<u64>,
    #[structopt(long)]
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
