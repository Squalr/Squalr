use crate::command_executors::unprivileged_command_executor::UnprivilegedCommandExecutor;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandExecutor for ProjectItemsCommand {
    type ResponseType = UnprivilegedCommandResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandExecutor>::ResponseType {
        match self {
            ProjectItemsCommand::Add { project_items_add_request } => project_items_add_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::Activate {
                project_items_activate_request,
            } => project_items_activate_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::Create { project_items_create_request } => project_items_create_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::Delete { project_items_delete_request } => project_items_delete_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::List { project_items_list_request } => project_items_list_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::Move { project_items_move_request } => project_items_move_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::Rename { project_items_rename_request } => project_items_rename_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
            ProjectItemsCommand::Reorder { project_items_reorder_request } => project_items_reorder_request
                .execute(engine_unprivileged_state)
                .to_engine_response(),
        }
    }
}
