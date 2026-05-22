use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLinePluginsCommand {
    List {
        #[structopt(flatten)]
        plugin_list_request: CommandLinePluginListRequest,
    },
    SetEnabled {
        #[structopt(flatten)]
        plugin_set_enabled_request: CommandLinePluginSetEnabledRequest,
    },
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLinePluginListRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePluginSetEnabledRequest {
    #[structopt(long = "plugin-id")]
    pub plugin_id: String,
    #[structopt(long = "enabled")]
    pub is_enabled: bool,
}

impl From<CommandLinePluginsCommand> for api::commands::plugins::plugins_command::PluginsCommand {
    fn from(command: CommandLinePluginsCommand) -> Self {
        match command {
            CommandLinePluginsCommand::List { plugin_list_request } => Self::List {
                plugin_list_request: plugin_list_request.into(),
            },
            CommandLinePluginsCommand::SetEnabled { plugin_set_enabled_request } => Self::SetEnabled {
                plugin_set_enabled_request: plugin_set_enabled_request.into(),
            },
        }
    }
}

impl From<CommandLinePluginListRequest> for api::commands::plugins::list::plugin_list_request::PluginListRequest {
    fn from(_: CommandLinePluginListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLinePluginSetEnabledRequest> for api::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest {
    fn from(request: CommandLinePluginSetEnabledRequest) -> Self {
        Self {
            plugin_id: request.plugin_id,
            is_enabled: request.is_enabled,
        }
    }
}
