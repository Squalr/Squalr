use crate::commands::plugins::list::plugin_list_response::PluginListResponse;
use crate::commands::plugins::set_enabled::plugin_set_enabled_response::PluginSetEnabledResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PluginsResponse {
    List { plugin_list_response: PluginListResponse },
    SetEnabled { plugin_set_enabled_response: PluginSetEnabledResponse },
}

impl TypedPrivilegedCommandResponse for PluginsResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Plugins(self.clone())
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Plugins(plugins_response) = response {
            Ok(plugins_response)
        } else {
            Err(response)
        }
    }
}
