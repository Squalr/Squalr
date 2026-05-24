use crate::commands::plugins::plugins_response::PluginsResponse;
use crate::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use crate::plugins::PluginState;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginSetOrderResponse {
    pub plugins: Vec<PluginState>,
    pub opened_process_info: Option<OpenedProcessInfo>,
    pub default_plugin_ids: Vec<String>,
    pub did_update: bool,
}

impl TypedPrivilegedCommandResponse for PluginSetOrderResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::Plugins(PluginsResponse::SetOrder {
            plugin_set_order_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::Plugins(PluginsResponse::SetOrder { plugin_set_order_response }) = response {
            Ok(plugin_set_order_response)
        } else {
            Err(response)
        }
    }
}
