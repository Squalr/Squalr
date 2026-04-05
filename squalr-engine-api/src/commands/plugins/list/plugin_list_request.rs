use crate::commands::plugins::list::plugin_list_response::PluginListResponse;
use crate::commands::plugins::plugins_command::PluginsCommand;
use crate::commands::plugins::plugins_response::PluginsResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize, Default)]
pub struct PluginListRequest {}

impl PrivilegedCommandRequest for PluginListRequest {
    type ResponseType = PluginListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Plugins(PluginsCommand::List {
            plugin_list_request: self.clone(),
        })
    }
}

impl From<PluginListResponse> for PluginsResponse {
    fn from(plugin_list_response: PluginListResponse) -> Self {
        PluginsResponse::List { plugin_list_response }
    }
}
