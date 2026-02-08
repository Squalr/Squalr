use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::commands::{
    privileged_command_response::TypedPrivilegedCommandResponse, settings::memory::list::memory_settings_list_response::MemorySettingsListResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemorySettingsResponse {
    Set {
        memory_settings_set_response: MemorySettingsSetResponse,
    },
    List {
        memory_settings_list_response: MemorySettingsListResponse,
    },
}

impl TypedPrivilegedCommandResponse for MemorySettingsResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::Memory { memory_settings_response }) = response {
            Ok(memory_settings_response)
        } else {
            Err(response)
        }
    }
}
