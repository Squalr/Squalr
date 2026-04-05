use crate::commands::plugins::plugins_command::PluginsCommand;
use crate::commands::plugins::plugins_response::PluginsResponse;
use crate::commands::plugins::set_enabled::plugin_set_enabled_response::PluginSetEnabledResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PluginSetEnabledRequest {
    #[structopt(long = "plugin-id")]
    pub plugin_id: String,
    #[structopt(long = "enabled")]
    pub is_enabled: bool,
}

impl PrivilegedCommandRequest for PluginSetEnabledRequest {
    type ResponseType = PluginSetEnabledResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Plugins(PluginsCommand::SetEnabled {
            plugin_set_enabled_request: self.clone(),
        })
    }
}

impl From<PluginSetEnabledResponse> for PluginsResponse {
    fn from(plugin_set_enabled_response: PluginSetEnabledResponse) -> Self {
        PluginsResponse::SetEnabled { plugin_set_enabled_response }
    }
}
