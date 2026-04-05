use crate::commands::plugins::plugins_response::PluginsResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::plugins::PluginState;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginListResponse {
    pub plugins: Vec<PluginState>,
    pub opened_process_info: Option<OpenedProcessInfo>,
}

impl TypedPrivilegedCommandResponse for PluginListResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Plugins(PluginsResponse::List {
            plugin_list_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Plugins(PluginsResponse::List { plugin_list_response }) = response {
            Ok(plugin_list_response)
        } else {
            Err(response)
        }
    }
}
