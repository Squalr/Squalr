use crate::commands::plugins::plugins_command::PluginsCommand;
use crate::commands::plugins::plugins_response::PluginsResponse;
use crate::commands::plugins::set_order::plugin_set_order_response::PluginSetOrderResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginSetOrderRequest {
    pub plugin_ids: Vec<String>,
}

impl PrivilegedCommandRequest for PluginSetOrderRequest {
    type ResponseType = PluginSetOrderResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Plugins(PluginsCommand::SetOrder {
            plugin_set_order_request: self.clone(),
        })
    }
}

impl From<PluginSetOrderResponse> for PluginsResponse {
    fn from(plugin_set_order_response: PluginSetOrderResponse) -> Self {
        PluginsResponse::SetOrder { plugin_set_order_response }
    }
}
