use crate::command_executors::engine_command_executor::EngineCommandExecutor;
use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::engine_command_response::{EngineCommandResponse, TypedEngineCommandResponse};
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use std::sync::Arc;

impl EngineCommandExecutor for ProjectItemsCommand {
    type ResponseType = EngineCommandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandExecutor>::ResponseType {
        match self {
            ProjectItemsCommand::Activate {
                project_items_activate_request,
            } => project_items_activate_request
                .execute(engine_privileged_state)
                .to_engine_response(),
            ProjectItemsCommand::List { project_items_list_request } => project_items_list_request
                .execute(engine_privileged_state)
                .to_engine_response(),
        }
    }
}
