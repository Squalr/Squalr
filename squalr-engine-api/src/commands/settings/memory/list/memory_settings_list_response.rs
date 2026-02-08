use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::settings_error::SettingsError;
use crate::commands::settings::settings_response::SettingsResponse;
use crate::structures::settings::memory_settings::MemorySettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemorySettingsListResponse {
    pub memory_settings: Result<MemorySettings, SettingsError>,
}

impl TypedPrivilegedCommandResponse for MemorySettingsListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: MemorySettingsResponse::List {
                memory_settings_list_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: MemorySettingsResponse::List { memory_settings_list_response },
        }) = response
        {
            Ok(memory_settings_list_response)
        } else {
            Err(response)
        }
    }
}
