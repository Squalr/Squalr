use crate::command_executors::privileged_command_executor::PrivilegedCommandExecutor;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::plugins::plugins_command::PluginsCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use std::sync::Arc;

impl PrivilegedCommandExecutor for PluginsCommand {
    type ResponseType = PrivilegedCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandExecutor>::ResponseType {
        match self {
            PluginsCommand::List { plugin_list_request } => plugin_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            PluginsCommand::SetEnabled { plugin_set_enabled_request } => plugin_set_enabled_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
