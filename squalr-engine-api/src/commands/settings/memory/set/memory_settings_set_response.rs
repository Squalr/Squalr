use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::settings::memory::memory_settings_response::MemorySettingsResponse;
use crate::commands::settings::settings_response::SettingsResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemorySettingsSetResponse {}

impl TypedPrivilegedCommandResponse for MemorySettingsSetResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: MemorySettingsResponse::Set {
                memory_settings_set_response: self.clone(),
            },
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Settings(SettingsResponse::Memory {
            memory_settings_response: MemorySettingsResponse::Set { memory_settings_set_response },
        }) = response
        {
            Ok(memory_settings_set_response)
        } else {
            Err(response)
        }
    }
}
