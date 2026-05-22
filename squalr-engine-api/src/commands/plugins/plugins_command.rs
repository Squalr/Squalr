use crate::commands::plugins::list::plugin_list_request::PluginListRequest;
use crate::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PluginsCommand {
    List { plugin_list_request: PluginListRequest },
    SetEnabled { plugin_set_enabled_request: PluginSetEnabledRequest },
}
