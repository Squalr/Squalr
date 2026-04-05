use crate::commands::plugins::list::plugin_list_request::PluginListRequest;
use crate::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum PluginsCommand {
    List {
        #[structopt(flatten)]
        plugin_list_request: PluginListRequest,
    },
    SetEnabled {
        #[structopt(flatten)]
        plugin_set_enabled_request: PluginSetEnabledRequest,
    },
}
